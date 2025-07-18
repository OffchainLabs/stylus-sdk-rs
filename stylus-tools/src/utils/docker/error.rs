// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Error handling for Docker usage.

use std::{io, str};

const CANNOT_CONNECT: &str = "Cannot connect to the Docker daemon";

#[derive(Debug, thiserror::Error)]
pub enum DockerError {
    #[error("Failed to execute Docker command: {0}")]
    CommandExecution(io::Error),
    #[error("Wait failed: {0}")]
    WaitFailure(io::Error),
    #[error("Invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Failed to read Docker command stderr {0}")]
    StderrUtf8(str::Utf8Error),
    #[error("Docker not running")]
    CannotConnect(String),
    #[error("Docker error: {0}")]
    Docker(String),
}

impl DockerError {
    pub(crate) fn from_stderr(stderr: Vec<u8>) -> Self {
        let stderr = match str::from_utf8(&stderr) {
            Ok(s) => s.to_string(),
            Err(err) => return Self::StderrUtf8(err),
        };
        if stderr.contains(CANNOT_CONNECT) {
            Self::CannotConnect(stderr)
        } else {
            Self::Docker(stderr)
        }
    }
}
