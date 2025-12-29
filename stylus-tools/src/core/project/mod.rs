// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::path::PathBuf;

pub use hash::{hash_project, ProjectHash};
pub use init::{init_contract, init_workspace, InitError};
pub use new::{new_contract, new_workspace};

pub mod contract;
pub mod workspace;

mod hash;
mod init;
mod new;

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
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Error processing path: {0}")]
    Glob(#[from] glob::GlobError),

    #[error("{0}")]
    Command(#[from] crate::error::CommandError),
    #[error("rust toolchain error: {0}")]
    Toolchain(#[from] crate::utils::toolchain::ToolchainError),

    #[error("Unable to read directory {dir}: {1}", dir = .0.display())]
    DirectoryRead(PathBuf, std::io::Error),
    #[error("Error finding file in {dir}: {1}", dir = .0.display())]
    DirectoryEntry(PathBuf, std::io::Error),
    #[error("Failed to read glob pattern '{0}': {1}")]
    GlobPattern(String, glob::PatternError),
    #[error("failed to open file {filename}: {1}", filename = .0.display())]
    FileOpen(PathBuf, std::io::Error),
    #[error("failed to read file {filename}: {1}", filename = .0.display())]
    FileRead(PathBuf, std::io::Error),
}
