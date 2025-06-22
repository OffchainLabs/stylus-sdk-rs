// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::path::PathBuf;

use alloy::primitives::Address;

use crate::error::CargoStylusResult;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// The WASM to check (defaults to any found in the current directory).
    #[arg(long)]
    wasm_file: Option<PathBuf>,
    /// Where to deploy and activate the contract (defaults to a random address).
    #[arg(long)]
    contract_address: Option<Address>,
}

pub fn exec(_args: Args) -> CargoStylusResult {
    Ok(())
}
