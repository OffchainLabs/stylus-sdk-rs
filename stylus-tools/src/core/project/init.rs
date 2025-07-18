// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Initialize Stylus workspaces and contracts.

use std::path::Path;

use crate::utils::{
    cargo::{self, manifest::ManifestMut},
    create_dir_if_dne,
    stylus_sdk::contract_dependencies,
};

/// Comment which is added to the `opt-level` key within the `[profile.release]` section of
/// `Cargo.toml` for contracts.
///
/// This provides a potential hint to users looking to optimize their contract binary size.
const OPT_LEVEL_COMMENT: &str = r#"
# If you need to reduce the binary size, it is advisable to try other
# optimization levels, such as "s" and "z"
"#;

/// Errors which may occur from initializing Stylus projects.
#[derive(Debug, thiserror::Error)]
pub enum InitError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("cargo metadata error: {0}")]
    CargoMetadata(#[from] cargo_metadata::Error),
    #[error("toml edit error: {0}")]
    TomlEdit(#[from] toml_edit::TomlError),

    #[error("cargo manifest error: {0}")]
    CargoManifest(#[from] crate::utils::cargo::manifest::CargoManifestError),
    #[error("{0}")]
    Command(#[from] crate::error::CommandError),
}

/// Initialize a Stylus contract in an existing Rust crate.
pub fn init_contract(path: impl AsRef<Path>) -> Result<(), InitError> {
    let path = path.as_ref();

    // Update Cargo.toml
    init_package_manifest(path)?;

    // Add files from template
    copy_from_template_if_dne!(
        "../../templates/contract" -> path,
        "src/lib.rs",
        "src/main.rs",
        "rust-toolchain.toml",
        "Stylus.toml",
    );

    Ok(())
}

/// Initialize a Stylus workspace in an existing directory.
pub fn init_workspace(path: impl AsRef<Path>) -> Result<(), InitError> {
    let path = path.as_ref();

    // Create standard directories
    create_dir_if_dne(path.join("contracts"))?;
    create_dir_if_dne(path.join("crates"))?;

    // Add files from template
    copy_from_template_if_dne!(
        "../../templates/workspace" -> path,
        "Cargo.toml",
        "rust-toolchain.toml",
        "Stylus.toml",
    );

    Ok(())
}

/// Initialize a contract's Cargo.toml.
///
/// Takes a path to the package directory.
fn init_package_manifest(path: impl AsRef<Path>) -> Result<(), InitError> {
    // Add required dependencies
    cargo::add(&path, contract_dependencies()?)?;

    // Parse existing manifest to add default configs
    // TODO: get this from cargo metadata
    let mut manifest = ManifestMut::read(path.as_ref().join("Cargo.toml"))?;
    manifest.lib().extend_crate_type(["lib", "cdylib"])?;

    // Add [profile.release] section
    let mut release = manifest.profile("release")?;
    release.set_default("codegen-units", 1);
    release.set_default("strip", true);
    release.set_default("lto", true);
    release.set_default("panic", "abort");
    if release.set_default("opt-level", 3) {
        release.add_comment("opt-level", OPT_LEVEL_COMMENT)?;
    }

    // Write the modified Cargo.toml file
    manifest.write()?;
    Ok(())
}
