// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::io::Write;

use cargo_metadata::Package;
use tempfile::NamedTempFile;

use crate::utils::{
    docker::{self, validate_host, DockerError},
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

    // TODO: How to know to call `stylus` or `stylus-beta`?
    let mut args = vec!["cargo".to_string(), "stylus".to_string()];
    for arg in command_line.into_iter() {
        args.push(arg);
    }
    // Use package source if available, otherwise use current working directory
    let source = package
        .source
        .as_ref()
        .map(|s| s.repr.to_owned())
        .unwrap_or_else(|| {
            std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .to_string_lossy()
                .to_string()
        });

    docker::run_in_container(&image_name, &source, args)?;
    Ok(())
}

/// Returns the image name
fn create_image(
    cargo_stylus_version: &str,
    toolchain_version: &str,
) -> Result<String, DockerError> {
    let name = image_name(cargo_stylus_version, toolchain_version);

    // First, check if image exists locally
    if docker::image_exists_locally(&name)? {
        info!(@grey, "Using local image {name}");
        return Ok(name);
    }
    info!(@grey, "Building Docker image for Rust toolchain {toolchain_version}");

    // Second, check if base image exists on Docker Hub. If not, we fail early since
    // docker build will fail trying to pull such image
    let base_image = format!("offchainlabs/cargo-stylus-base:{cargo_stylus_version}");
    info!(@grey, "Checking if base image exists on Docker Hub: {base_image}");

    if !docker::image_exists_on_hub(&base_image)? {
        return Err(DockerError::ImageNotFound(
            base_image,
            cargo_stylus_version.to_owned(),
        ));
    }

    info!(@grey, "Image exists, building container with base image: {base_image}");

    // Create temporary Dockerfile
    let dockerfile_content = format!(
        r#"\
            ARG BUILD_PLATFORM=linux/amd64
            FROM --platform=${{BUILD_PLATFORM}} {base_image} AS base
            RUN rustup toolchain install {toolchain_version}-x86_64-unknown-linux-gnu 
            RUN rustup default {toolchain_version}-x86_64-unknown-linux-gnu
            RUN rustup target add wasm32-unknown-unknown
            RUN rustup component add rust-src --toolchain {toolchain_version}-x86_64-unknown-linux-gnu
        "#
    );

    // Write to temporary file (automatically cleaned up when dropped)
    let temp_file = NamedTempFile::new().map_err(|e| DockerError::Io(e))?;
    temp_file
        .as_file()
        .write_all(dockerfile_content.as_bytes())
        .map_err(|e| DockerError::Io(e))?;

    // Build using the temporary file
    docker::cmd::build_with_file(&name, temp_file.path())?;
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
