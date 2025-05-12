// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::path::Path;

use crate::core::project::{new_contract, new_workspace, ProjectKind};

/// Create a new Stylus contract or workspace.
pub fn new(path: impl AsRef<Path>, kind: ProjectKind) -> eyre::Result<()> {
    match kind {
        ProjectKind::Contract => new_contract(path)?,
        ProjectKind::Workspace => new_workspace(path)?,
    };
    Ok(())
}
