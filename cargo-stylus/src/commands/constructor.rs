// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use crate::{
    common_args::{ProjectArgs, ReflectionArgs},
    error::CargoStylusResult,
};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(flatten)]
    project: ProjectArgs,
    #[command(flatten)]
    reflection: ReflectionArgs,
}

pub fn exec(args: Args) -> CargoStylusResult {
    let config = args.reflection.config();
    for contract in args.project.contracts()? {
        contract.print_constructor(&config)?;
    }
    Ok(())
}
