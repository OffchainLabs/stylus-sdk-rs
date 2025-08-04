// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::path::PathBuf;

use alloy::primitives::Address;
use stylus_tools::{
    core::{check::CheckConfig, network, project::ProjectHash},
    ops,
};

use crate::{
    common_args::{ActivationArgs, BuildArgs, ProviderArgs},
    error::CargoStylusResult,
};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// The WASM to check (defaults to any found in the current directory).
    #[arg(long)]
    wasm_file: Option<PathBuf>,
    /// Where to deploy and activate the contract (defaults to a random address).
    // TODO: how will this work for multiple contracts
    #[arg(long)]
    contract_address: Option<Address>,

    #[command(flatten)]
    activation: ActivationArgs,
    #[command(flatten)]
    build: BuildArgs,
    #[command(flatten)]
    provider: ProviderArgs,
}

pub async fn exec(args: Args) -> CargoStylusResult {
    network::check_endpoint(&args.provider.endpoint)?;
    let provider = args.provider.build_provider().await?;
    let config = CheckConfig {
        activation: args.activation.into_config(),
        ..Default::default()
    };
    if let Some(wasm_file) = args.wasm_file {
        ops::check_wasm_file(
            wasm_file,
            ProjectHash::default(),
            args.contract_address,
            &config,
            &provider,
        )
        .await?;
    } else {
        ops::check_workspace(&config, &provider).await?;
    };

    Ok(())
}
