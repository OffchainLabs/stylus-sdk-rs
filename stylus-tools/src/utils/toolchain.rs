// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::{fs, io, path::PathBuf};

use cargo_metadata::Package;
use serde::Deserialize;

const TOOLCHAIN_FILE_NAME: &str = "rust-toolchain.toml";
const INVALID_CHANNELS: &[&str] = &["stable", "nightly", "beta"];

const NOT_FOUND_MSG: &str =
    "expected to find a rust-toolchain.toml file in project directory \
    to specify your Rust toolchain for reproducible verification. The channel in your project's rust-toolchain.toml's \
    toolchain section must be a specific version e.g., '1.80.0' or 'nightly-YYYY-MM-DD'. \
    To ensure reproducibility, it cannot be a generic channel like 'stable', 'nightly', or 'beta'. Read more about \
    the toolchain file in https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file or see \
    the file in https://github.com/OffchainLabs/stylus-hello-world for an example";
const INVALID_CHANNEL_MSG: &str =
    "the channel in your project's rust-toolchain.toml's toolchain section must be a specific version e.g., '1.80.0' or 'nightly-YYYY-MM-DD'. \
    To ensure reproducibility, it cannot be a generic channel like 'stable', 'nightly', or 'beta'";

pub fn get_toolchain_channel(package: &Package) -> Result<String, ToolchainError> {
    let dir = package.manifest_path.parent().unwrap();
    let toolchain_path = find_toolchain_file(dir)?;
    let toolchain_file_contents = fs::read_to_string(toolchain_path)?;
    let toolchain_toml: ToolchainFile = toml::from_str(&toolchain_file_contents)?;
    let channel = toolchain_toml.toolchain.channel;

    // Reject "stable" and "nightly" channels specified alone
    if INVALID_CHANNELS.contains(&channel.as_str()) {
        return Err(ToolchainError::InvalidChannel);
    }

    // Parse the Rust version from the toolchain project, only allowing alphanumeric chars and dashes.
    let channel = channel
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '.')
        .collect();

    Ok(channel)
}

pub fn find_toolchain_file(dir: impl Into<PathBuf>) -> Result<PathBuf, ToolchainError> {
    let mut path = dir.into();
    while !path.join(TOOLCHAIN_FILE_NAME).exists() {
        path = path.parent().ok_or(ToolchainError::NotFound)?.to_path_buf();
    }
    Ok(path.join(TOOLCHAIN_FILE_NAME))
}

#[derive(Debug, Deserialize)]
pub struct ToolchainFile {
    toolchain: Toolchain,
}

#[derive(Debug, Deserialize)]
pub struct Toolchain {
    channel: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ToolchainError {
    #[error("error reading rust-toolchain.toml: {0}")]
    Io(#[from] io::Error),

    #[error("invalid rust-toolchain.toml: {0}")]
    TomlDeserialize(#[from] toml::de::Error),

    #[error("{INVALID_CHANNEL_MSG}")]
    InvalidChannel,
    #[error("{NOT_FOUND_MSG}")]
    NotFound,
}
