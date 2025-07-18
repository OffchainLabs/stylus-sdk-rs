// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

// TODO: structured version types?
fn _image_name(cargo_stylus_version: &str, toolchain_version: &str) -> String {
    format!("cargo-stylus-base-{cargo_stylus_version}-toolchain-{toolchain_version}")
}

/// Verify that the OS supports verifiable builds.
pub fn verify_os() -> Result<(), VerifyOsError> {
    // TODO: consider using sysinfo, used elsewhere more
    let os_type = sys_info::os_type().map_err(VerifyOsError::UnsupportedOs)?;
    if os_type == "Windows" {
        let kernel_version = sys_info::os_release().map_err(VerifyOsError::UnsupportedRelease)?;
        if kernel_version.contains("microsoft") || kernel_version.contains("WSL") {
            info!(@grey, "Detected Windows Linux Subsystem host");
            Ok(())
        } else {
            Err(VerifyOsError::WindowsWithoutWsl)
        }
    } else {
        Ok(())
    }
}

/// Error returned with info about why the current OS does not support verification.
#[derive(Debug, thiserror::Error)]
pub enum VerifyOsError {
    #[error("Unable to determine host OS type")]
    UnsupportedOs(sys_info::Error),
    #[error("Unable to determine kernel version")]
    UnsupportedRelease(sys_info::Error),
    #[error("Windows without WSL not supported")]
    WindowsWithoutWsl,
}
