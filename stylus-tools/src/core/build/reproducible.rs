// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::io::Write;

use cargo_metadata::Package;

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
    let cargo_stylus_version =
        cargo_stylus_version.unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());
    let image_name = create_image(&cargo_stylus_version, &toolchain_channel)?;

    let mut args = vec!["cargo".to_string(), "stylus".to_string()];
    for arg in command_line.into_iter() {
        args.push(arg);
    }
    docker::run_in_container(&image_name, package.source.clone().unwrap().repr, args)?;
    Ok(())
}

/// Returns the image name
fn create_image(
    cargo_stylus_version: &str,
    toolchain_version: &str,
) -> Result<String, DockerError> {
    let name = image_name(cargo_stylus_version, toolchain_version);
    if image_exists(&name)? {
        return Ok(name);
    }
    println!(
        "Building Docker image for Rust toolchain {}",
        toolchain_version
    );
    let mut build = docker::cmd::build(&name)?;
    write!(
        build.file(),
        "\
            ARG BUILD_PLATFORM=linux/amd64
            FROM --platform=${{BUILD_PLATFORM}} offchainlabs/cargo-stylus-base:{} AS base
            RUN rustup toolchain install {}-x86_64-unknown-linux-gnu 
            RUN rustup default {}-x86_64-unknown-linux-gnu
            RUN rustup target add wasm32-unknown-unknown
            RUN rustup component add rust-src --toolchain {}-x86_64-unknown-linux-gnu
        ",
        cargo_stylus_version,
        toolchain_version,
        toolchain_version,
        toolchain_version,
    )?;
    build.wait()?;
    Ok(name)
}

fn image_name(cargo_stylus_version: &str, toolchain_version: &str) -> String {
    format!("cargo-stylus-base-{cargo_stylus_version}-toolchain-{toolchain_version}")
}

#[derive(Debug, thiserror::Error)]
pub enum ReproducibleBuildError {
    #[error("docker error: {0}")]
    Docker(#[from] DockerError),
    #[error("toolchain error: {0}")]
    Toolchain(#[from] ToolchainError),
}
