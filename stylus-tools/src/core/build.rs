// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::{path::PathBuf, process::Stdio};

use cargo_metadata::MetadataCommand;
use escargot::Cargo;

use crate::core::project::contract::Contract;

const WASM_TARGET: &str = "wasm32-unknown-unknown";
const OPT_LEVEL_Z_CONFIG: &str = "profile.release.opt-level='z'";
const UNSTABLE_FLAGS: &[&str] = &[
    "build-std=std,panic_abort",
    "build-std-features=panic_immediate_abort",
];

#[derive(Clone, Debug, Default)]
pub struct BuildConfig {
    pub opt_level: OptLevel,
    pub features: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub enum OptLevel {
    #[default]
    S,
    Z,
}

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("cargo error: {0}")]
    Cargo(#[from] escargot::error::CargoError),
    #[error("cargo metadata error: {0}")]
    CargoMetadata(#[from] cargo_metadata::Error),

    #[error("{0}")]
    Toolchain(#[from] crate::utils::toolchain::ToolchainError),

    #[error("build did not generate wasm file")]
    NoWasmFound,
    #[error("failed to execute cargo build")]
    FailedToExecute,
    #[error("cargo build command failed")]
    CargoBuildFailed,
}

pub fn build_contract(contract: &Contract, config: &BuildConfig) -> Result<PathBuf, BuildError> {
    info!(@grey, "Building project with Cargo.toml version: {}", contract.version());

    let mut cmd = Cargo::new()
        .args(["build", "--lib", "--locked", "--release"])
        .args(["--target", WASM_TARGET]);
    if !config.features.is_empty() {
        cmd = cmd.args(["--features", &config.features.join(" ")]);
    }
    if !contract.stable() {
        cmd = cmd.args(UNSTABLE_FLAGS.iter().flat_map(|flag| ["-Z", flag]));
    }
    if matches!(config.opt_level, OptLevel::Z) {
        cmd = cmd.args(["--config", OPT_LEVEL_Z_CONFIG]);
    }

    let status = cmd
        .into_command()
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|_| BuildError::FailedToExecute)?;
    if !status.success() {
        return Err(BuildError::CargoBuildFailed);
    }

    let metadata = MetadataCommand::new().exec()?;
    let wasm_path = metadata
        .target_directory
        .join(WASM_TARGET)
        .join("release")
        .join("deps")
        .join(format!("{}.wasm", contract.name()));
    if !wasm_path.exists() {
        return Err(BuildError::NoWasmFound);
    }

    Ok(wasm_path.into())
}
