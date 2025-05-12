// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::path::Path;

use crate::core::project::{init_contract, init_workspace, ProjectKind};

/// Initialize a Stylus contract or workspace in an existing directory.
pub fn init(path: impl AsRef<Path>, kind: ProjectKind) -> eyre::Result<()> {
    match kind {
        ProjectKind::Contract => init_contract(path)?,
        ProjectKind::Workspace => init_workspace(path)?,
    }
    Ok(())
}
