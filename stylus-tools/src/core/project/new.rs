// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::{fs, path::Path};

use super::init::{init_contract, init_workspace, project_name};
use crate::{
    core::project::InitError,
    utils::{cargo, git},
    Result,
};

/// Create a new Stylus contract.
pub fn new_contract(path: impl AsRef<Path>, sdk_path: Option<&Path>) -> Result<(), InitError> {
    let path = path.as_ref();
    let project = project_name(path)?;

    let result = new_contract_inner(path, &project, sdk_path);
    if let Err(ref e) = result {
        eprintln!(
            "\nerror: failed to create Stylus project at '{}': {e}\n\
             \n\
             The project directory may have been left in a partially initialized state.\n\
             This can happen if your cargo-stylus version is incompatible with the\n\
             current SDK dependencies. Try updating:\n\
             \n    cargo install --force cargo-stylus\n",
            path.display()
        );
    }
    result
}

fn new_contract_inner(
    path: &Path,
    project: &str,
    sdk_path: Option<&Path>,
) -> Result<(), InitError> {
    // Initialize a Rust package with cargo
    cargo::new(path)?;
    // Upgrade the Rust package into a Stylus contract
    init_contract(path, sdk_path)?;

    // Remove the generated "src/lib.rs" and generate the new one
    fs::remove_file(path.join("src").join("lib.rs"))?;
    copy_from_template_if_dne!(
        (project),
        "templates/contract" -> path,
        "src/lib.rs",
    );

    // Ensure Cargo.lock exists so that `cargo stylus check --locked` works
    cargo::generate_lockfile(path)?;

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
