// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::path::PathBuf;

use stylus_tools::ops;

use crate::error::CargoStylusResult;

#[derive(Debug, clap::Args)]
pub struct Args {
    input: PathBuf,
    out_dir: PathBuf,
}

pub fn exec(args: Args) -> CargoStylusResult {
    ops::c_gen(args.input, args.out_dir)?;
    Ok(())
}
