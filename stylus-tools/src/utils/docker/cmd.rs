// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Run commands from the Docker CLI.

use std::{
    ffi::OsStr,
    path::Path,
    process::{Command, Stdio},
    str,
};

use super::{error::DockerError, json};

const DOCKER_PROGRAM: &str = "docker";

/// Build a Docker image using a Dockerfile from an existing file.
pub fn build_with_file(tag: &str, dockerfile_path: &Path) -> Result<(), DockerError> {
    info!(@grey, "Building Docker image: {} (using Dockerfile: {})", tag, dockerfile_path.display());

    let mut child = Command::new(DOCKER_PROGRAM)
        .arg("build")
        .args(["--tag", tag])
        .args(["--file", dockerfile_path.to_str().unwrap()])
        .arg(".")
        .stdout(Stdio::inherit()) // Stream stdout to terminal
        .stderr(Stdio::inherit()) // Stream stderr to terminal
        .spawn()
        .map_err(DockerError::CommandExecution)?;

    let status = child.wait().map_err(DockerError::WaitFailure)?;

    if !status.success() {
        return Err(DockerError::Docker(format!(
            "Docker build failed with exit code: {}",
            status.code().unwrap_or(-1)
        )));
    }

    info!(@grey, "Docker image built successfully: {}", tag);
    Ok(())
}

/// Check if a Docker image exists on Docker Hub.
pub fn image_exists_on_hub(image_name: &str) -> Result<bool, DockerError> {
    // Use docker manifest inspect to check if the image exists on the registry
    let output = Command::new(DOCKER_PROGRAM)
        .arg("manifest")
        .arg("inspect")
        .arg(image_name)
        .output()
        .map_err(DockerError::CommandExecution)?;

    // If the command succeeds, the image exists
    let exists = output.status.success();

    Ok(exists)
}

/// Check if a specific Docker image exists locally.
/// Returns true if the exact image:tag combination exists locally.
pub fn image_exists_locally(image_name: &str) -> Result<bool, DockerError> {
    let output = Command::new(DOCKER_PROGRAM)
        .arg("images")
        .args(["--format", "json"])
        .arg(image_name)
        .output()
        .map_err(DockerError::CommandExecution)?;

    let success = output.status.success();
    if !success {
        return Err(DockerError::from_stderr(output.stderr));
    }

    // Parse the JSON output to check if any images match the exact image:tag
    let images: Result<Vec<json::Image>, _> = output
        .stdout
        .split(|b| *b == b'\n')
        .filter(|slice| !slice.is_empty()) // Filter out empty lines
        .map(|slice| serde_json::from_slice(slice).map_err(|error| DockerError::Json(error)))
        .collect::<Result<Vec<_>, _>>();

    let images = images?;

    // Check if any image matches the exact repository:tag combination
    let exists = images.iter().any(|image| {
        let full_name = if image.tag == "<none>" {
            image.repository.clone()
        } else {
            format!("{}:{}", image.repository, image.tag)
        };
        full_name == image_name
    });

    Ok(exists)
}

/// Run a command in a Docker container.
pub fn run(
    image: &str,
    network: Option<&str>,
    volumes: &[(&str, &str)],
    workdir: Option<&str>,
    args: impl IntoIterator<Item = impl AsRef<OsStr>>,
) -> Result<(), DockerError> {
    // TODO: builder pattern
    // TODO: --mount instead of --volume
    // TODO: check return code
    let mut cmd = Command::new(DOCKER_PROGRAM);
    cmd.arg("run");
    if let Some(network) = network {
        cmd.args(["--network", network]);
    }
    if let Some(workdir) = workdir {
        cmd.args(["--workdir", workdir]);
    }
    for (host_path, container_path) in volumes {
        cmd.args(["--volume", &format!("{host_path}:{container_path}")]);
    }
    cmd.arg(image);
    cmd.args(args);
    cmd.spawn()
        .map_err(DockerError::CommandExecution)?
        .wait()
        .map_err(DockerError::WaitFailure)?;
    Ok(())
}
