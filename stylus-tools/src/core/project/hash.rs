// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use super::{ProjectConfig, ProjectError};

pub type ProjectHash = [u8; 32];

pub fn hash_project(config: &ProjectConfig) -> Result<ProjectHash, ProjectError> {
    todo!()
}
