// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::{iter, path::PathBuf};

use alloy::primitives::Address;
use itertools::izip;
use stylus_tools::{
    core::{network, project::ProjectHash},
    ops,
};

use crate::{
    common_args::{ActivationArgs, BuildArgs, CheckArgs, ProjectArgs, ProviderArgs},
    error::CargoStylusResult,
};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// The WASM to check (defaults to any found in the current directory).
    #[arg(long)]
    wasm_file: Vec<PathBuf>,
    /// Where to deploy and activate the contract for wasm file (defaults to a random address).
    #[arg(long)]
    wasm_file_address: Vec<Address>,
    /// Where to deploy and activate the contract (defaults to a random address).
    #[arg(long)]
    contract_address: Vec<Address>,

    #[command(flatten)]
    activation: ActivationArgs,
    #[command(flatten)]
    build: BuildArgs,
    #[command(flatten)]
    check: CheckArgs,
    #[command(flatten)]
    project: ProjectArgs,
    #[command(flatten)]
    provider: ProviderArgs,
}

pub async fn exec(args: Args) -> CargoStylusResult {
    network::check_endpoint(&args.provider.endpoint)?;
    let provider = args.provider.build_provider().await?;
    let config = args.check.config(&args.activation);

    for (wasm_file, address) in izip!(
        args.wasm_file,
        args.wasm_file_address
            .into_iter()
            .map(Some)
            .chain(iter::repeat(None))
    ) {
        ops::check_wasm_file(
            wasm_file,
            ProjectHash::default(),
            address,
            &config,
            &provider,
        )
        .await?;
    }

    for (contract, address) in izip!(
        args.project.contracts()?,
        args.contract_address
            .into_iter()
            .map(Some)
            .chain(iter::repeat(None))
    ) {
        contract.check(address, &config, &provider).await?;
    }

    Ok(())
}
