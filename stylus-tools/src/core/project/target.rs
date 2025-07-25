// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![allow(dead_code)]

use super::{
    contract::Contract,
    workspace::{Workspace, WorkspaceError},
};

/// Some commands or functions may operate on either a workspace, or a specific set of contracts.
#[derive(Debug)]
pub enum Target {
    Contracts(Vec<Contract>),
    Workspace(Workspace),
}

impl Target {
    pub fn current_workspace() -> Result<Self, WorkspaceError> {
        let workspace = Workspace::current()?;
        Ok(Self::Workspace(workspace))
    }
}
