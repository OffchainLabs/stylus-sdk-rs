// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Utilities for working with docker.

use std::{ffi::OsStr, path::Path};

pub use error::DockerError;

pub mod cmd;
pub mod json;

mod error;

pub fn validate_host() -> Result<(), DockerError> {
    let os_type = sys_info::os_type().map_err(|_| DockerError::UnableToDetermineOsType)?;
    if os_type == "Windows" {
        // Check for WSL environment
        let kernel_version =
            sys_info::os_release().map_err(|_| DockerError::UnableToDetermineKernelVersion)?;
        if kernel_version.contains("microsoft") || kernel_version.contains("WSL") {
            greyln!("Detected Windows Linux Subsystem host");
        } else {
            return Err(DockerError::WSLRequired);
        }
    }
    Ok(())
}

pub fn image_exists(image_name: &str) -> Result<bool, DockerError> {
    Ok(!cmd::images(image_name)?.is_empty())
}

pub fn run_in_container(
    image_name: &str,
    dir: impl AsRef<Path>,
    args: impl IntoIterator<Item = impl AsRef<OsStr>>,
) -> Result<(), DockerError> {
    cmd::run(
        image_name,
        Some("host"),
        &[(dir.as_ref().to_str().unwrap(), "/source")],
        Some("/source"),
        args,
    )?;
    Ok(())
}
