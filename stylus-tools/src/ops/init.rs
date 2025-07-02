// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::path::Path;

use crate::{
    core::project::ProjectKind,
    utils::{cargo, create_dir_if_dne},
    Result,
};

/// Initialize a Stylus contract or workspace in an existing directory.
pub fn init(path: impl AsRef<Path>, kind: ProjectKind) -> Result<()> {
    match kind {
        ProjectKind::Contract => init_contract(path),
        ProjectKind::Workspace => init_workspace(path),
    }
}

/// Initialize a Stylus contract in an existing Rust crate.
pub fn init_contract(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();

    cargo::add(path, [concat!("stylus-sdk@", env!("CARGO_PKG_VERSION"))])?;

    copy_from_template_if_dne!(
        "../templates/contract" -> path,
        "src/lib.rs",
        "src/main.rs",
        "Stylus.toml",
    );

    Ok(())
}

/// Initialize a Stylus workspace in an existing directory.
pub fn init_workspace(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();

    create_dir_if_dne(path.join("contracts"))?;
    create_dir_if_dne(path.join("crates"))?;

    copy_from_template_if_dne!(
        "../templates/workspace" -> path,
        "Cargo.toml",
        "rust-toolchain.toml",
        "Stylus.toml",
    );

    Ok(())
}
