// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::path::PathBuf;

use alloy::primitives::Address;
use stylus_tools::{
    core::{build::Build, project::ProjectHash},
    ops,
};

use crate::{
    common_args::{BuildArgs, DataFeeArgs, ProviderArgs},
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
    build: BuildArgs,
    #[command(flatten)]
    data_fee: DataFeeArgs,
    #[command(flatten)]
    provider: ProviderArgs,
}

pub async fn exec(args: Args) -> CargoStylusResult {
    let provider = args.provider.build_provider().await?;
    if let Some(wasm_file) = args.wasm_file {
        ops::check_wasm_file(
            wasm_file,
            ProjectHash::default(),
            args.contract_address,
            args.data_fee.bump_percent,
            &provider,
        )
        .await?;
    } else {
        ops::check(args.data_fee.bump_percent, &provider).await?;
    };

    Ok(())
}
