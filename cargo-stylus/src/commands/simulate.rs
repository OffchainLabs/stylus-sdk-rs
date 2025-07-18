// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use crate::error::CargoStylusResult;

#[derive(Debug, clap::Args)]
pub struct Args {}

pub fn exec(_args: Args) -> CargoStylusResult {
    Ok(())
}
