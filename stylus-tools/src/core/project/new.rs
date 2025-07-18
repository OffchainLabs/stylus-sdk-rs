// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::{fs, path::Path};

use super::{init_contract, init_workspace};
use crate::{
    core::project::InitError,
    utils::{cargo, git},
    Result,
};

/// Create a new Stylus contract.
pub fn new_contract(path: impl AsRef<Path>) -> Result<(), InitError> {
    // Initialize a Rust package with cargo
    cargo::new(&path)?;
    // Remove the generated "src/lib.rs" so init_contract() will create a new one
    fs::remove_file(path.as_ref().join("src").join("lib.rs"))?;
    // Upgrade the Rust package into a Stylus contract
    init_contract(&path)?;
    Ok(())
}

/// Create a new Stylus workspace.
pub fn new_workspace(path: impl AsRef<Path>) -> Result<(), InitError> {
    // Create a new git repo at the given path
    git::init(Some(&path))?;
    // Upgrade the new repository into a Stylus workspace
    init_workspace(&path)?;
    Ok(())
}
