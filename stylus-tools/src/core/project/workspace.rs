// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::{
    env,
    path::{Path, PathBuf},
};

use super::contract::Contract;
use crate::{cargo, error::Result};

/// Workspace root for a Stylus project.
///
/// The workspace is defined by a `Stylus.toml` which is in a cargo workspace directory alongside the
/// `Cargo.toml` manifest file.
#[derive(Debug)]
pub struct Workspace {
    cargo_metadata: cargo::metadata::Metadata,
}

impl Workspace {
    /// Create reference to an existing Stylus workspace.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let cargo_metadata = cargo::cmd::metadata(path)?;
        Ok(Self { cargo_metadata })
    }

    /// Get a reference to the current Stylus workspace.
    pub fn current() -> Result<Self> {
        Self::new(env::current_dir()?)
    }

    /// Create a new Stylus workspace.
    ///
    /// Used by the `cargo new --workspace` command.
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        std::fs::create_dir(path)?;
        std::fs::create_dir(path.join("contracts"))?;
        std::fs::create_dir(path.join("crates"))?;

        copy_from_template!(
            "../../templates/workspace" -> path,
            "Cargo.toml",
            "rust-toolchain.toml",
            "Stylus.toml",
        );

        Self::new(path)
    }

    /// Check validity of Stylus contracts within a workspace.
    pub fn check(&self) -> Result<()> {
        for contract in self.contracts()? {
            contract.check();
        }
        Ok(())
    }

    pub fn contracts(&self) -> Result<impl Iterator<Item = Contract> + '_> {
        Ok(self
            .cargo_metadata
            .workspace_members()
            .cloned()
            .filter_map(|cargo_package| Contract::try_from(cargo_package).ok()))
    }

    pub fn default_contracts(&self) -> Result<impl Iterator<Item = Contract> + '_> {
        Ok(self
            .cargo_metadata
            .workspace_default_members()
            .cloned()
            .filter_map(|cargo_package| Contract::try_from(cargo_package).ok()))
    }
}
