// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::{cmp::Ordering, io::Write};

use cargo_metadata::{semver::Version, MetadataCommand, Package};
use tempfile::NamedTempFile;

use crate::{
    core::optimize::{BinaryenVersion, MIN_CARGO_STYLUS_VERSION},
    utils::{
        docker::{self, validate_host, DockerError},
        toolchain::{get_toolchain_channel, ToolchainError},
    },
};

pub fn run_reproducible(
    package: &Package,
    cargo_stylus_version: Option<String>,
    wasm_opt_version: Option<BinaryenVersion>,
    command_line: impl IntoIterator<Item = String>,
) -> Result<(), ReproducibleBuildError> {
    validate_host()?;
    let toolchain_channel = get_toolchain_channel(package)?;
    greyln!(
        "Running reproducible Stylus command with toolchain {}",
        toolchain_channel.mint()
    );

    let selected_cargo_stylus_version = select_stylus_version(cargo_stylus_version)?;

    // A [wasm-opt] recipe is only honored by base images from MIN_CARGO_STYLUS_VERSION onward.
    // Refuse rather than silently deploy unoptimized bytes that would still verify green.
    if wasm_opt_version.is_some() {
        let min = Version::parse(MIN_CARGO_STYLUS_VERSION)
            .expect("MIN_CARGO_STYLUS_VERSION is a valid semver version");
        if selected_cargo_stylus_version < min {
            return Err(ReproducibleBuildError::WasmOptUnsupported {
                selected: selected_cargo_stylus_version,
                min,
            });
        }
    }

    let image_name = create_image(
        &selected_cargo_stylus_version,
        &toolchain_channel,
        wasm_opt_version.as_ref(),
    )?;

    // Currently only calling cargo stylus is supported (not cargo stylus-beta for instance)
    let mut args = vec!["cargo".to_string(), "stylus".to_string()];
    for arg in command_line.into_iter() {
        args.push(arg);
    }
    // Mount the workspace root so that workspace-level Cargo.toml, Cargo.lock,
    // and path dependencies are all available inside the container.
    let metadata = MetadataCommand::new().no_deps().exec()?;
    let source = metadata.workspace_root.to_string();

    docker::run_in_container(&image_name, &source, args)?;
    Ok(())
}

/// Returns the image name
fn create_image(
    cargo_stylus_version: &Version,
    toolchain_version: &str,
    wasm_opt_version: Option<&BinaryenVersion>,
) -> Result<String, DockerError> {
    let name = image_name(cargo_stylus_version, toolchain_version, wasm_opt_version);

    // First, check if image exists locally
    if docker::image_exists_locally(&name)? {
        info!(@grey, "Using local image {name}");
        return Ok(name);
    }
    info!(@grey, "Building Docker image for Rust toolchain {toolchain_version}");

    // Second, check if base image exists locally or on Docker Hub. If not, we fail
    // early since docker build will fail trying to pull such image.
    let base_image = format!("offchainlabs/cargo-stylus-base:{cargo_stylus_version}");

    if docker::image_exists_locally(&base_image)? {
        info!(@grey, "Using local base image: {base_image}");
    } else {
        info!(@grey, "Checking if base image exists on Docker Hub: {base_image}");
        if !docker::image_exists_on_hub(&base_image)? {
            return Err(DockerError::ImageNotFound(
                base_image.clone(),
                cargo_stylus_version.to_string(),
            ));
        }
    }

    info!(@grey, "Image exists, building container with base image: {base_image}");

    let binaryen_layer = wasm_opt_version.map(binaryen_layer).unwrap_or_default();

    // Create temporary Dockerfile
    let dockerfile_content = format!(
        r#"\
            ARG BUILD_PLATFORM=linux/amd64
            FROM --platform=${{BUILD_PLATFORM}} {base_image} AS base
            RUN rustup toolchain install {toolchain_version}-x86_64-unknown-linux-gnu
            RUN rustup default {toolchain_version}-x86_64-unknown-linux-gnu
            RUN rustup target add wasm32-unknown-unknown
            RUN rustup component add rust-src --toolchain {toolchain_version}-x86_64-unknown-linux-gnu
{binaryen_layer}        "#
    );

    // Write to temporary file (automatically cleaned up when dropped)
    let temp_file = NamedTempFile::new().map_err(DockerError::Io)?;
    temp_file
        .as_file()
        .write_all(dockerfile_content.as_bytes())
        .map_err(DockerError::Io)?;

    // Build using the temporary file
    docker::cmd::build_with_file(&name, temp_file.path())?;
    Ok(name)
}

/// Dockerfile fragment installing the pinned Binaryen `wasm-opt` so the reproducible build applies
/// the same optimization step as verification. The release tarball is downloaded over HTTPS and
/// verified against its published SHA-256 checksum before extraction. The tarball bundles
/// `wasm-opt` alongside its supporting library files, so the whole tree is extracted to /opt and
/// its bin/ added to PATH.
///
/// `BinaryenVersion` is digits-only by construction, so interpolating it into this `RUN` cannot
/// inject shell metacharacters.
fn binaryen_layer(version: &BinaryenVersion) -> String {
    let tarball = format!("binaryen-version_{version}-x86_64-linux.tar.gz");
    let base_url =
        format!("https://github.com/WebAssembly/binaryen/releases/download/version_{version}");
    format!(
        r#"            RUN cd /tmp \
                && curl -fsSL --proto '=https' --tlsv1.2 -O {base_url}/{tarball} \
                && curl -fsSL --proto '=https' --tlsv1.2 -O {base_url}/{tarball}.sha256 \
                && sha256sum -c {tarball}.sha256 \
                && tar -xzf {tarball} -C /opt \
                && rm {tarball} {tarball}.sha256
            ENV PATH="/opt/binaryen-version_{version}/bin:${{PATH}}"
"#
    )
}

fn image_name(
    cargo_stylus_version: &Version,
    toolchain_version: &str,
    wasm_opt_version: Option<&BinaryenVersion>,
) -> String {
    let base = format!("cargo-stylus-base-{cargo_stylus_version}-toolchain-{toolchain_version}");
    match wasm_opt_version {
        // Distinct cached image per Binaryen version so different pins don't collide.
        Some(version) => format!("{base}-binaryen-{version}"),
        None => base,
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ReproducibleBuildError {
    #[error("docker error: {0}")]
    Docker(#[from] DockerError),
    #[error("toolchain error: {0}")]
    Toolchain(#[from] ToolchainError),
    #[error("cargo metadata error: {0}")]
    CargoMetadata(#[from] cargo_metadata::Error),
    #[error(
        "cargo-stylus {selected} does not support the [wasm-opt] table (added in {min}); \
select version {min} or newer with --cargo-stylus-version, or upgrade cargo-stylus if no \
version was selected explicitly"
    )]
    WasmOptUnsupported { selected: Version, min: Version },
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

    info!(@blue, "Using cargo-stylus version: {selected_stylus_version}");

    Ok(selected_stylus_version)
}

#[cfg(test)]
mod tests {
    use cargo_metadata::semver::Version;

    use super::{binaryen_layer, image_name, select_stylus_version, BinaryenVersion};

    fn binaryen(version: &str) -> BinaryenVersion {
        version.parse().unwrap()
    }

    #[test]
    fn image_name_encodes_binaryen_version() {
        let version = Version::parse("0.10.9").unwrap();
        assert_eq!(
            image_name(&version, "1.91.0", None),
            "cargo-stylus-base-0.10.9-toolchain-1.91.0"
        );
        assert_eq!(
            image_name(&version, "1.91.0", Some(&binaryen("131"))),
            "cargo-stylus-base-0.10.9-toolchain-1.91.0-binaryen-131"
        );
        // Different Binaryen pins must yield different cache keys, or a bumped pin would silently
        // reuse the image (and the wasm-opt) of the previous version.
        assert_ne!(
            image_name(&version, "1.91.0", Some(&binaryen("131"))),
            image_name(&version, "1.91.0", Some(&binaryen("132")))
        );
    }

    /// The rendered Binaryen layer is never executed in tests or CI (the integration harness runs
    /// wasm-opt in-process), so pin its load-bearing lines here: the release URL, the checksum
    /// verification, and the PATH entry.
    #[test]
    fn binaryen_layer_renders_install_commands() {
        let layer = binaryen_layer(&binaryen("131"));
        assert!(layer.contains(
            "https://github.com/WebAssembly/binaryen/releases/download/version_131\
/binaryen-version_131-x86_64-linux.tar.gz"
        ));
        assert!(layer.contains("sha256sum -c binaryen-version_131-x86_64-linux.tar.gz.sha256"));
        assert!(layer.contains(r#"ENV PATH="/opt/binaryen-version_131/bin:${PATH}""#));
    }

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
