// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::path::{Path, PathBuf};

use cargo_metadata::{MetadataCommand, Package};

use crate::Result;

/// Build a Rust project to WASM and return the path to the compiled WASM file.
pub fn build_contract(package: &Package) -> Result<PathBuf> {
    todo!("return path")
}

/// Build contracts in a workspace
pub fn build_workspace(cargo_manifest_path: impl AsRef<Path>) -> Result<Vec<Result<PathBuf>>> {
    let metadata = MetadataCommand::new()
        .manifest_path(cargo_manifest_path.as_ref())
        .exec()?;
    Ok(metadata
        .workspace_default_packages()
        .into_iter()
        .map(build_contract)
        .collect())
}
