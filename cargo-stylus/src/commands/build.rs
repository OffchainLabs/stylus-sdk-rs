// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use stylus_tools::ops;

use crate::{common_args::BuildArgs, error::CargoStylusResult};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(flatten)]
    build: BuildArgs,
}

pub fn exec(args: Args) -> CargoStylusResult {
    let config = args.build.into_config();
    ops::build_workspace(&config)?;
    Ok(())
}
