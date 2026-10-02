// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

pub mod reproducible;

use std::{
    path::{Path, PathBuf},
    process::Stdio,
};

use cargo_metadata::MetadataCommand;
use escargot::Cargo;

use crate::core::project::contract::Contract;

const WASM_TARGET: &str = "wasm32-unknown-unknown";
const OPT_LEVEL_Z_CONFIG: &str = "profile.release.opt-level='z'";

// Nightlies before `nightly-2025-10-05` enable immediate-abort panics through the
// `panic_immediate_abort` build-std feature.
const LEGACY_UNSTABLE_FLAGS: &[&str] = &[
    "build-std=std,panic_abort",
    "build-std-features=panic_immediate_abort",
];

// Nightlies from `nightly-2025-10-05` onward replaced that feature with the `immediate-abort` panic
// strategy, which Cargo can only be told about through the release profile. Setting it here is what
// keeps the project's `target.<triple>.rustflags` (stack size, target features, ...) intact, which
// `RUSTFLAGS` would override. `-Z panic-immediate-abort` opts into the unstable Cargo feature.
const IMMEDIATE_ABORT_BUILD_STD_FLAGS: &[&str] = &["build-std=std,panic_abort"];
const IMMEDIATE_ABORT_CARGO_FLAG: &str = "panic-immediate-abort";
const IMMEDIATE_ABORT_PROFILE_CONFIG: &str = "profile.release.panic='immediate-abort'";

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
        .args(["--target", WASM_TARGET])
        .args(["--package", contract.package.name.as_str()]);
    if !config.features.is_empty() {
        cmd = cmd.args(["--features", &config.features.join(" ")]);
    }
    let immediate_abort = !contract.stable() && contract.supports_immediate_abort();
    if !contract.stable() {
        if immediate_abort {
            cmd = cmd
                .args(
                    IMMEDIATE_ABORT_BUILD_STD_FLAGS
                        .iter()
                        .flat_map(|flag| ["-Z", flag]),
                )
                .args(["-Z", IMMEDIATE_ABORT_CARGO_FLAG])
                .args(["--config", IMMEDIATE_ABORT_PROFILE_CONFIG]);
        } else {
            cmd = cmd.args(LEGACY_UNSTABLE_FLAGS.iter().flat_map(|flag| ["-Z", flag]));
        }
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
    let wasm_path = find_wasm(metadata.target_directory.as_std_path(), contract.name())
        .ok_or(BuildError::NoWasmFound)?;

    Ok(wasm_path)
}

/// Locates the compiled wasm for a contract.
///
/// Older Cargo versions place the artifact in `target/<target>/release/deps/`, while newer ones
/// drop the `deps` directory and uplift it to `target/<target>/release/` directly.
fn find_wasm(target_dir: &Path, name: &str) -> Option<PathBuf> {
    let release_dir = target_dir.join(WASM_TARGET).join("release");
    let file_name = format!("{name}.wasm");
    [
        release_dir.join("deps").join(&file_name),
        release_dir.join(&file_name),
    ]
    .into_iter()
    .find(|path| path.exists())
}
