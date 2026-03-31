// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::Stdio,
};

use cargo_metadata::MetadataCommand;
use escargot::{format::Message, Cargo, CommandMessages};

use crate::{
    error::{CommandError, CommandFailure},
    Result,
};

pub mod manifest;

/// Source for a dependency added via `cargo add`.
#[derive(Debug)]
pub enum DepSource {
    /// A versioned dependency from the registry (e.g. "0.10.2").
    Version(String),
    /// A local path dependency.
    Path(PathBuf),
}

pub fn add(
    project_path: impl AsRef<Path>,
    sources: impl IntoIterator<Item = (String, DepSource)>,
) -> Result<(), CommandError> {
    let mut version_args = Vec::new();
    let mut path_deps = Vec::new();

    for (name, source) in sources {
        match source {
            DepSource::Version(req) => version_args.push(format!("{name}@{req}")),
            DepSource::Path(path) => path_deps.push(path),
        }
    }

    // Add all version-based deps in one invocation
    if !version_args.is_empty() {
        let mut cmd = Cargo::new().into_command();
        let output = cmd
            .arg("add")
            .args(&version_args)
            .current_dir(&project_path)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()?;
        CommandFailure::check("cargo add", output)?;
    }

    // Path deps must be added one at a time
    for dep_path in path_deps {
        let mut cmd = Cargo::new().into_command();
        let output = cmd
            .arg("add")
            .arg("--path")
            .arg(&dep_path)
            .current_dir(&project_path)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()?;
        CommandFailure::check("cargo add --path", output)?;
    }

    Ok(())
}

pub fn generate_lockfile(path: impl AsRef<Path>) -> Result<(), CommandError> {
    let mut cmd = Cargo::new().into_command();
    let output = cmd
        .arg("generate-lockfile")
        .current_dir(path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;
    CommandFailure::check("cargo generate-lockfile", output)?;
    Ok(())
}

pub fn clean() -> Result<(), CommandError> {
    let mut cmd = Cargo::new().into_command();
    let output = cmd
        .arg("clean")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;
    CommandFailure::check("cargo clean", output)?;
    Ok(())
}

pub fn new(path: impl AsRef<Path>) -> Result<(), CommandError> {
    let mut cmd = Cargo::new().into_command();
    let output = cmd
        .args(["new", "--lib"])
        .arg(path.as_ref())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;
    CommandFailure::check("cargo new", output)?;
    Ok(())
}

pub fn version() -> Result<String, CommandError> {
    let mut cmd = Cargo::new().into_command();
    let output = cmd
        .arg("version")
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()?;
    let version = CommandFailure::check("cargo version", output)?;
    Ok(version)
}

pub fn is_workspace_root(path: impl AsRef<Path>) -> Result<bool> {
    let path = fs::canonicalize(path)?;
    let metadata = MetadataCommand::new().exec()?;
    let workspace_root = fs::canonicalize(metadata.workspace_root)?;
    Ok(path == workspace_root)
}

pub fn parse_messages_for_filename(
    messages: CommandMessages,
    target: impl AsRef<OsStr>,
) -> escargot::error::CargoResult<Option<PathBuf>> {
    let target = target.as_ref();
    let maybe_path = messages.into_iter().find_map(|msg| {
        if let Message::CompilerArtifact(artifact) = msg.ok()?.decode().ok()? {
            artifact
                .filenames
                .into_iter()
                .find_map(|path| (path.file_name() == Some(target)).then(|| path.to_path_buf()))
        } else {
            None
        }
    });
    Ok(maybe_path)
}
