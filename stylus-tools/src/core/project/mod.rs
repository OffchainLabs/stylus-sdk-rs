// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

pub use hash::{hash_project, ProjectHash};
pub use init::{init_contract, init_workspace, InitError};
pub use new::{new_contract, new_workspace};

pub mod contract;
pub mod workspace;

mod hash;
mod init;
mod new;
mod target;

#[derive(Debug, Default)]
pub struct ProjectConfig {
    pub source_file_patterns: Vec<String>,
}

#[derive(Debug)]
pub enum ProjectKind {
    Contract,
    Workspace,
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    #[error("{0}")]
    Command(#[from] crate::error::CommandError),
}
