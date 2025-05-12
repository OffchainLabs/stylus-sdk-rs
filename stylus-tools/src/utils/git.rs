// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::{
    path::Path,
    process::{Command, Stdio},
};

use crate::error::CommandError;

const GIT: &str = "git";

/// Call `git init` as a subprocess.
pub fn init(dir: Option<impl AsRef<Path>>) -> Result<(), CommandError> {
    let mut cmd = Command::new(GIT);
    cmd.arg("init")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    if let Some(dir) = dir {
        cmd.arg(dir.as_ref());
    }
    let status = cmd.status()?;
    if !status.success() {
        todo!("handle error");
    }
    Ok(())
}
