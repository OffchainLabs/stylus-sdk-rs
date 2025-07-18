// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::path::PathBuf;

use stylus_tools::ops;

use crate::error::CargoStylusResult;

#[derive(Debug, clap::Args)]

pub struct Args {
    /// The output file (defaults to stdout).
    #[arg(long)]
    output: Option<PathBuf>,
    /// Write a JSON ABI instead using solc. Requires solc.
    #[arg(long)]
    json: bool,
    /// Rust crate's features list. Required to include feature specific abi.
    #[arg(long)]
    rust_features: Option<Vec<String>>,
}

pub fn exec(args: Args) -> CargoStylusResult {
    ops::export_abi(args.output, args.json, args.rust_features)?;
    Ok(())
}
