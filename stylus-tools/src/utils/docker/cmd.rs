// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Run commands from the Docker CLI.

#![allow(dead_code)]

use std::{
    process::{Command, Stdio},
    str,
};

use super::{error::DockerError, json};

const DOCKER_PROGRAM: &str = "docker";

/// Start a Docker build.
pub fn build(tag: &str) {
    let _child = Command::new(DOCKER_PROGRAM)
        .arg("build")
        .args(["--tag", tag])
        .arg(".")
        .args(["--file", "-"])
        .stdin(Stdio::piped())
        .spawn();
}

/// List local Docker images.
///
/// We currently only use this with a repository specified.
pub fn images(repository: &str) -> Result<Vec<json::Image>, DockerError> {
    let output = Command::new(DOCKER_PROGRAM)
        .arg("images")
        .args(["--format", "json"])
        .arg(repository)
        .output()
        .map_err(DockerError::CommandExecution)?;

    if !output.status.success() {
        return Err(DockerError::from_stderr(output.stderr));
    }

    output
        .stdout
        .split(|b| *b == b'\n')
        .map(|slice| serde_json::from_slice(slice).map_err(Into::into))
        .collect()
}

/// Run a command in a Docker container.
pub fn run(
    image: &str,
    network: Option<&str>,
    volumes: &[(&str, &str)],
    workdir: Option<&str>,
) -> Result<(), DockerError> {
    // TODO: builder pattern
    // TODO: --mount instead of --volume
    // TODO: check return code
    let mut cmd = Command::new(DOCKER_PROGRAM);
    cmd.args(["run", image]);
    if let Some(network) = network {
        cmd.args(["--network", network]);
    }
    if let Some(workdir) = workdir {
        cmd.args(["--workdir", workdir]);
    }
    for (host_path, container_path) in volumes {
        cmd.args(["--volume", &format!("{host_path}:{container_path}")]);
    }
    cmd.spawn()
        .map_err(DockerError::CommandExecution)?
        .wait()
        .map_err(DockerError::WaitFailure)?;
    Ok(())
}
