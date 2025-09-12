// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::{fs, path::Path};

use serde::de::DeserializeOwned;

pub mod contract;
pub mod workspace;

/// Filename for Stylus.toml manifest files (both workspace and contract)
pub const FILENAME: &str = "Stylus.toml";

#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("toml read error: {0}")]
    TomlRead(#[from] toml::de::Error),

    #[error("missing Stylus.toml")]
    Missing,
}

pub fn load<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T, ManifestError> {
    if !path.as_ref().exists() {
        return Err(ManifestError::Missing);
    }

    let contents = fs::read_to_string(path)?;
    let manifest = toml::from_str(&contents)?;
    Ok(manifest)
}
