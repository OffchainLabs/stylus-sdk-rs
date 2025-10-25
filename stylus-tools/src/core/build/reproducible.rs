// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::{cmp::Ordering, io::Write};

use cargo_metadata::{semver::Version, Package};

use crate::utils::{
    docker::{self, image_exists, validate_host, DockerError},
    toolchain::{get_toolchain_channel, ToolchainError},
};

pub fn run_reproducible(
    package: &Package,
    cargo_stylus_version: Option<String>,
    command_line: impl IntoIterator<Item = String>,
) -> Result<(), ReproducibleBuildError> {
    validate_host()?;
    let toolchain_channel = get_toolchain_channel(package)?;
    greyln!(
        "Running reproducible Stylus command with toolchain {}",
        toolchain_channel.mint()
    );

    let selected_cargo_stylus_version = select_stylus_version(cargo_stylus_version)?;
    let image_name = create_image(&selected_cargo_stylus_version, &toolchain_channel)?;

    let mut args = vec!["cargo".to_string(), "stylus".to_string()];
    for arg in command_line.into_iter() {
        args.push(arg);
    }
    docker::run_in_container(&image_name, package.source.clone().unwrap().repr, args)?;
    Ok(())
}

/// Returns the image name
fn create_image(
    cargo_stylus_version: &Version,
    toolchain_version: &str,
) -> Result<String, DockerError> {
    let name = image_name(cargo_stylus_version, toolchain_version);
    if image_exists(&name)? {
        return Ok(name);
    }
    println!("Building Docker image for Rust toolchain {toolchain_version}");
    let mut build = docker::cmd::build(&name)?;
    write!(
        build.file(),
        "\
            ARG BUILD_PLATFORM=linux/amd64
            FROM --platform=${{BUILD_PLATFORM}} offchainlabs/cargo-stylus-base:{} AS base
            RUN rustup toolchain install {toolchain_version}-x86_64-unknown-linux-gnu 
            RUN rustup default {toolchain_version}-x86_64-unknown-linux-gnu
            RUN rustup target add wasm32-unknown-unknown
            RUN rustup component add rust-src --toolchain {toolchain_version}-x86_64-unknown-linux-gnu
        ",
        cargo_stylus_version.to_string()
    )?;
    build.wait()?;
    Ok(name)
}

fn image_name(cargo_stylus_version: &Version, toolchain_version: &str) -> String {
    format!(
        "cargo-stylus-base-{}-toolchain-{toolchain_version}",
        cargo_stylus_version.to_string()
    )
}

#[derive(Debug, thiserror::Error)]
pub enum ReproducibleBuildError {
    #[error("docker error: {0}")]
    Docker(#[from] DockerError),
    #[error("toolchain error: {0}")]
    Toolchain(#[from] ToolchainError),
}

/// Returns the selected cargo_stylus_version if `cargo_stylus_version` is Some, otherwise returns
/// the current version which is defined by env var CARGO_PKG_VERSION. In case there's a version
/// mismatch between user cargo_stylus_version and cargo `CARGO_PKG_VERSION` we display appropriate
/// warnings to let the user know the run might not be reproducible.
fn select_stylus_version(
    cargo_stylus_version: Option<String>,
) -> Result<Version, ReproducibleBuildError> {
    let current_version_str = env!("CARGO_PKG_VERSION");
    let mut selected_stylus_version =
        Version::parse(current_version_str).expect("Failed to parse CARGO_PKG_VERSION");

    if let Some(user_version_str) = cargo_stylus_version {
        match Version::parse(&user_version_str) {
            Ok(user_version) => {
                match user_version.cmp(&selected_stylus_version) {
                    Ordering::Less => {
                        warn!(@yellow, "############### OLDER VERSION WARNING ###############");
                        warn!(@yellow, "You have selected cargo-stylus version {}.", user_version_str);
                        warn!(@yellow, "This is OLDER than the current tool's version {}.", current_version_str);
                        warn!(@yellow, "Using an older, potentially buggy version is not recommended.");
                        warn!(@yellow, "Please consider using version {} or newer.", current_version_str);
                        warn!(@yellow, "#####################################################");
                    }

                    Ordering::Greater => {
                        warn!(@yellow, "############### VERSION MISMATCH WARNING ###############");
                        warn!(@yellow, "Selected cargo stylus version {} is NEWER than current cargo stylus version {}", user_version_str, current_version_str);
                        warn!(@yellow, "This may result in a reproducible build that does not match the original build.");
                        warn!(@yellow, "Please use the same cargo stylus version as the original build.");
                        warn!(@yellow, "########################################################");
                    }

                    Ordering::Equal => {
                        // Versions match. No warning needed.
                    }
                }

                selected_stylus_version = user_version;
            }
            Err(e) => {
                warn!(@red, "Invalid version string provided: '{}'. Error: {}", user_version_str, e);
                warn!(@red, "Defaulting to current version {}.", current_version_str);
            }
        }
    }

    info!(@blue, "Using cargo-stylus version: {}", selected_stylus_version.to_string());

    Ok(selected_stylus_version)
}

#[cfg(test)]
mod tests {
    use super::select_stylus_version;
    use cargo_metadata::semver::Version;

    #[test]
    fn test_select_stylus_version() {
        let current_version_str = env!("CARGO_PKG_VERSION");
        let selected_stylus_version = Version::parse(current_version_str).unwrap();

        // Assert that we get system's cargo stylus version if None is passed in
        let chosen_version = select_stylus_version(None).unwrap();
        assert_eq!(selected_stylus_version, chosen_version);

        // Assert we get user selected cargo stylus version if it's greater than the system's cargo
        // stylus version
        let mut greater_version = selected_stylus_version.clone();
        greater_version.patch += 1;
        let chosen_version = select_stylus_version(Some(greater_version.to_string())).unwrap();
        assert_eq!(greater_version, chosen_version);

        // Assert we get user selected cargo stylus version if it's smaller than the system's cargo
        // stylus version
        let smaller_version = Version::parse("0.2.0-rc.0").unwrap();
        let chosen_version = select_stylus_version(Some(smaller_version.to_string())).unwrap();
        assert_eq!(smaller_version, chosen_version);
    }
}
