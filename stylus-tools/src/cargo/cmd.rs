// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Run cargo commands.

use std::{
    path::Path,
    process::{Command, Stdio},
};

use super::{
    build::{BuildConfig, JsonMessage, OptLevel},
    metadata,
};
use crate::error::Result;

const RUST_TARGET: &str = "wasm32-unknown-unknown";

/// Run `cargo build` for `cargo stylus` subcommands.
pub fn build<P: AsRef<Path>>(manifest_path: P, config: &BuildConfig) -> Result<Vec<JsonMessage>> {
    let mut cmd = Command::new("cargo");
    cmd.args([
        "build",
        "--lib",
        "--locked",
        "--release",
        "--message-format=json",
        &format!("--target={RUST_TARGET}"),
        "--manifest-path",
    ])
    .arg(manifest_path.as_ref());

    if let Some(features) = &config.features {
        cmd.arg(format!("--features={}", features));
    }

    if !config.stable {
        cmd.args([
            "-Z",
            "build-std=std,panic_abort",
            "-Z",
            "build-std-features=panic_immediate_abort",
        ]);
    }

    if matches!(config.opt_level, OptLevel::Z) {
        cmd.args(["--config", "profile.release.opt-level='z'"]);
    }

    let output = cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;

    // TODO: check output status
    /*
                    if !output.status.success() {
                        egreyln!("cargo build command failed");
                        process::exit(1);
                    }
    */
    // TODO: stream output?
    if !output.status.success() {
        todo!("output errors");
    }

    let messages = output
        .stdout
        .split(|b| *b == b'\n')
        .map(|line| serde_json::from_slice(line).map_err(Into::into))
        .collect::<Result<Vec<_>>>()?;
    Ok(messages)
}

/// Run `cargo metadata` and parse the result.
pub fn metadata<P: AsRef<Path>>(path: P) -> Result<metadata::Metadata> {
    let output = Command::new("cargo")
        .arg("metadata")
        .stderr(Stdio::inherit())
        .current_dir(path)
        .output()?;
    let metadata = serde_json::from_str(unsafe { &String::from_utf8_unchecked(output.stdout) })?;
    Ok(metadata)
}

/// Run `cargo new` for use in `cargo stylus new`.
pub fn new<P: AsRef<Path>>(path: P) -> Result<()> {
    // TODO: check return code and wrap error
    let _status = Command::new("cargo")
        .arg("new")
        .arg(path.as_ref())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    Ok(())
}
