// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![allow(dead_code)]

use std::{
    ffi::OsStr,
    fs, io,
    path::Path,
    process::{Command, Stdio},
};

pub fn command_exists(program: impl AsRef<OsStr>) -> bool {
    Command::new(program)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .arg("--version")
        .output()
        .map(|x| x.status.success())
        .unwrap_or_default()
}

/// Opens a file for writing, or stdout.
pub fn file_or_stdout(path: Option<impl AsRef<Path>>) -> io::Result<Box<dyn io::Write>> {
    Ok(match path {
        Some(file) => Box::new(fs::File::create(file)?),
        None => Box::new(io::stdout().lock()),
    })
}

pub fn host_arch() -> Result<String, rustc_host::Error> {
    rustc_host::from_cli()
}
