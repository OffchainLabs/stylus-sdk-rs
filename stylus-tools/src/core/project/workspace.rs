// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::collections::HashSet;

use cargo_metadata::{Metadata, MetadataCommand};
use cargo_util_schemas::manifest::PackageName;

use crate::core::{
    manifest::{self, workspace::WorkspaceManifest},
    project::contract::{Contract, ContractError},
};

/// Metadata for a Stylus project workspace
///
/// Stylus workspaces are built on top of cargo workspaces, adding a
/// `Stylus.toml` file alongside the usual `Cargo.toml` at the workspace root.
/// This file is used for Stylus-specific workspace configuration and is
/// required to use the workspace for Stylus contracts.
#[derive(Debug)]
pub struct Workspace {
    /// Stylus metadata
    pub manifest: WorkspaceManifest,
    /// Cargo metadata
    metadata: Metadata,
}

impl Workspace {
    /// Load the current Stylus workspace
    pub fn current() -> Result<Self, WorkspaceError> {
        let metadata = MetadataCommand::new().no_deps().exec()?;
        Self::try_from(metadata)
    }

    pub fn root_contract(&self) -> Option<Result<Contract, ContractError>> {
        self.metadata
            .root_package()
            .filter(|p| Contract::is_contract(p))
            .map(Contract::try_from)
    }

    pub fn contract(&self, contract_name: PackageName) -> Option<Result<Contract, ContractError>> {
        self.contracts().find(|c| match c {
            Ok(c) => c.package.name == contract_name,
            Err(_) => false,
        })
    }

    pub fn default_contracts(&self) -> impl Iterator<Item = Result<Contract, ContractError>> + '_ {
        self.metadata
            .workspace_default_packages()
            .into_iter()
            .filter(|p| Contract::is_contract(p))
            .map(Contract::try_from)
    }

    pub fn contracts(&self) -> impl Iterator<Item = Result<Contract, ContractError>> + '_ {
        self.metadata
            .workspace_packages()
            .into_iter()
            .filter(|p| Contract::is_contract(p))
            .map(Contract::try_from)
    }

    pub fn filter_contracts(
        &self,
        contract_names: impl Iterator<Item = PackageName>,
    ) -> Result<Vec<Contract>, ContractError> {
        let contract_names: HashSet<PackageName> = contract_names.collect();
        self.contracts()
            .filter(|c| match c {
                Ok(c) => contract_names.contains(&c.package.name),
                Err(_) => true,
            })
            .collect()
    }
}

impl TryFrom<Metadata> for Workspace {
    type Error = WorkspaceError;

    fn try_from(metadata: Metadata) -> Result<Self, Self::Error> {
        let manifest_path = metadata.workspace_root.join(manifest::FILENAME);
        let manifest = manifest::load(manifest_path)?;
        Ok(Self { manifest, metadata })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WorkspaceError {
    #[error("cargo metadata error: {0}")]
    CargoMetadata(#[from] cargo_metadata::Error),

    #[error("{0}")]
    Manifest(#[from] manifest::ManifestError),
}
