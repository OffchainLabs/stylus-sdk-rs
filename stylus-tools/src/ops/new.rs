// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::path::Path;

use crate::{
    core::project::ProjectKind,
    ops,
    utils::{cargo, git},
    Result,
};

/// Create a new Stylus contract or workspace.
pub fn new(path: impl AsRef<Path>, kind: ProjectKind) -> Result<()> {
    match kind {
        ProjectKind::Contract => new_contract(path)?,
        ProjectKind::Workspace => new_workspace(path)?,
    };
    Ok(())
}

/// Create a new Stylus contract.
pub fn new_contract(path: impl AsRef<Path>) -> Result<()> {
    // TODO: automatically place in <workspace>/contracts/
    cargo::new(&path)?;
    ops::init_contract(&path)?;
    Ok(())
}

/// Create a new Stylus workspace.
pub fn new_workspace(path: impl AsRef<Path>) -> Result<()> {
    git::init(Some(&path))?;
    ops::init_workspace(&path)?;
    Ok(())
}
