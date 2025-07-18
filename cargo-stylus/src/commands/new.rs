// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::path::PathBuf;

use eyre::eyre;
use stylus_tools::{core::project::ProjectKind, ops};

use crate::error::CargoStylusResult;

// TODO: --name
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Project name or path
    path: PathBuf,
    /// Create a new contract [default]
    #[arg(long)]
    contract: bool,
    /// Create a new workspace
    #[arg(long)]
    workspace: bool,
}

pub fn exec(args: Args) -> CargoStylusResult {
    let kind = match (args.contract, args.workspace) {
        // Default to `--contract`
        (true, false) | (false, false) => ProjectKind::Contract,
        (false, true) => ProjectKind::Workspace,
        (true, true) => Err(eyre!("cannot specify both --contract and --workspace"))?,
    };
    ops::new(args.path, kind)?;
    Ok(())
}
