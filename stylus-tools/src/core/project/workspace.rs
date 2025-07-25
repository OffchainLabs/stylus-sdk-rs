// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use cargo_metadata::{Metadata, MetadataCommand};

use super::contract::{Contract, ContractError};

#[derive(Debug)]
pub struct Workspace {
    metadata: Metadata,
}

impl Workspace {
    pub fn current() -> Result<Self, WorkspaceError> {
        let metadata = MetadataCommand::new().exec()?;
        Ok(Self { metadata })
    }

    pub fn contracts(&self) -> impl Iterator<Item = Result<Contract, ContractError>> + '_ {
        self.metadata
            .workspace_packages()
            .into_iter()
            .map(Contract::try_from)
            // skip packages that are not stylus contracts
            .filter(|res| !matches!(res, Err(ContractError::MissingStylusToml)))
    }

    pub fn default_contracts(&self) -> impl Iterator<Item = Result<Contract, ContractError>> + '_ {
        self.metadata
            .workspace_default_packages()
            .into_iter()
            .map(Contract::try_from)
            // skip packages that are not stylus contracts
            .filter(|res| !matches!(res, Err(ContractError::MissingStylusToml)))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WorkspaceError {
    #[error("cargo metadata error: {0}")]
    CargoMetadata(#[from] cargo_metadata::Error),
}
