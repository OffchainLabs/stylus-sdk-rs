// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::{ffi::OsStr, fs, path::Path, process::Stdio};

use cargo_metadata::MetadataCommand;
use escargot::Cargo;

use crate::Result;

pub fn new(path: impl AsRef<Path>) -> Result<()> {
    let mut cmd = Cargo::new().into_command();
    let _status = cmd
        .args(["new", "--lib"])
        .arg(path.as_ref())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    // TODO: check status
    Ok(())
}

pub fn add(
    path: impl AsRef<Path>,
    sources: impl IntoIterator<Item = impl AsRef<OsStr>>,
) -> Result<()> {
    let mut cmd = Cargo::new().into_command();
    let _status = cmd
        .arg("add")
        .args(sources)
        .current_dir(path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    // TODO: check status
    Ok(())
}

pub fn is_workspace_root(path: impl AsRef<Path>) -> Result<bool> {
    let path = fs::canonicalize(path)?;
    let metadata = MetadataCommand::new().exec()?;
    let workspace_root = fs::canonicalize(metadata.workspace_root)?;
    Ok(path == workspace_root)
}
