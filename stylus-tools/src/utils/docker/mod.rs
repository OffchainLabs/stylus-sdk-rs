// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Utilities for working with docker.

// TODO: pub here?
use error::DockerError;

pub mod cmd;
pub mod error;
pub mod json;

pub fn _image_exists(image_name: &str) -> Result<bool, DockerError> {
    Ok(!cmd::images(image_name)?.is_empty())
}
