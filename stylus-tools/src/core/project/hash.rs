// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use tiny_keccak::{Hasher, Keccak};

use crate::utils::cargo;

use super::{ProjectConfig, ProjectError};

pub type ProjectHash = [u8; 32];

pub fn hash_project(_config: &ProjectConfig) -> Result<ProjectHash, ProjectError> {
    let cargo_version = cargo::version()?;

    let mut keccak = Keccak::v256();
    keccak.update(cargo_version.as_bytes());
    // TODO: hash project files

    Ok(ProjectHash::default())
}
