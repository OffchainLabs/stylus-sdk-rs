// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use stylus_tools::{core::build::BuildConfig, ops};

use crate::{common_args::BuildArgs, error::CargoStylusResult};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg(long)]
    contract: Vec<String>,

    #[command(flatten)]
    build: BuildArgs,
}

pub fn exec(args: Args) -> CargoStylusResult {
    let config = BuildConfig {
        features: args.build.features,
        ..Default::default()
    };

    if args.contract.is_empty() {
        ops::build_workspace(&config)?;
    } else {
        for contract in args.contract {
            ops::build_contract(contract, &config)?;
        }
    }

    Ok(())
}
