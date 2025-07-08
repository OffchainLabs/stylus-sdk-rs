// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

pub use hash::{hash_project, ProjectHash};

mod hash;

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
pub enum ProjectError {}
