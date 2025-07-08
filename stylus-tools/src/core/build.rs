// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::path::PathBuf;

use escargot::CargoBuild;

use super::contract::Contract;
use crate::utils::cargo::parse_messages_for_filename;

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
    #[error("cargo error: {0}")]
    Cargo(#[from] escargot::error::CargoError),

    #[error("{0}")]
    Toolchain(#[from] crate::utils::toolchain::ToolchainError),

    #[error("build did not generate wasm file")]
    NoWasmFound,
}

pub fn build_contract(contract: &Contract, config: &BuildConfig) -> Result<PathBuf, BuildError> {
    info!(@grey, "Building project with Cargo.toml version: {}", contract.version());

    let mut cmd = CargoBuild::new()
        .args(["--lib", "--locked"])
        .release()
        .target(WASM_TARGET);
    if !config.features.is_empty() {
        cmd = cmd.features(config.features.join(" "));
    }
    if !contract.stable() {
        cmd = cmd.args(UNSTABLE_FLAGS.iter().flat_map(|flag| ["-Z", flag]));
    }
    if matches!(config.opt_level, OptLevel::Z) {
        cmd = cmd.args(["--config", OPT_LEVEL_Z_CONFIG]);
    }

    panic!("cmd: {:?}", cmd.into_command());
    /*
        let messages = cmd.exec()?;
        let wasm_file = parse_messages_for_filename(messages, format!("{}.wasm", contract.name()))?
            .ok_or(BuildError::NoWasmFound)?;
        Ok(wasm_file)
    */
}
