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

pub fn add(
    path: impl AsRef<Path>,
    sources: impl IntoIterator<Item = (String, String)>,
) -> Result<(), CommandError> {
    let mut cmd = Cargo::new().into_command();
    let output = cmd
        .arg("add")
        .args(
            sources
                .into_iter()
                .map(|(name, req)| format!("{name}@{req}")),
        )
        .current_dir(path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;
    CommandFailure::check("cargo add", output)?;
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
