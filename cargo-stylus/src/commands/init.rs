// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::path::PathBuf;

use eyre::eyre;
use stylus_tools::{core::project::ProjectKind, ops, utils::cargo};

use crate::error::CargoStylusResult;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Path to existing directory, cargo crate, or cargo workspace
    #[clap(default_value = ".")]
    path: PathBuf,
    /// Initialize a Stylus contract
    #[arg(long)]
    contract: bool,
    /// Initialize a Stylus workspace
    #[arg(long)]
    workspace: bool,
}

pub fn exec(args: Args) -> CargoStylusResult {
    let kind = match (args.contract, args.workspace) {
        (true, false) => ProjectKind::Contract,
        (false, true) => ProjectKind::Workspace,
        (false, false) => {
            if cargo::is_workspace_root(&args.path)? {
                ProjectKind::Workspace
            } else {
                ProjectKind::Contract
            }
        }
        (true, true) => return Err(eyre!("cannot specify both --contract and --workspace").into()),
    };
    ops::init(args.path, kind)?;
    Ok(())
}
