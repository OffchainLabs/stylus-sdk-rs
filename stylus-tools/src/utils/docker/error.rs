// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Error handling for Docker usage.

use std::{io, str};

const CANNOT_CONNECT: &str = "Cannot connect to the Docker daemon";

#[derive(Debug, thiserror::Error)]
pub enum DockerError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

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
    #[error(
        "Base Docker image '{0}' not found locally nor on Docker Hub.
            This usually means the version '{1}' is not available on Docker Hub.
            Available options:
            1. Visit https://hub.docker.com/r/offchainlabs/cargo-stylus-base/tags for all available versions
            2. Try using a stable version: cargo stylus --version <stable-version>
            3. Pull the image manually: docker pull {0}
            Common stable versions: 0.6.3, 0.6.2"
    )]
    ImageNotFound(String, String),

    #[error("unable to determine host OS type")]
    UnableToDetermineOsType,
    #[error("unable to determine kernel version")]
    UnableToDetermineKernelVersion,
    #[error(
        "Reproducible cargo stylus commands on Windows are only supported \
            in Windows Linux Subsystem (WSL). Please install within WSL. \
            To instead opt out of reproducible builds, add the --no-verify \
            flag to your commands."
    )]
    WSLRequired,
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
