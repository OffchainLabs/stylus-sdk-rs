// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use cargo_metadata::MetadataCommand;
use escargot::Cargo;

use crate::{core::project::contract::Contract, utils::sys};

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

/// Errors which can occur during the build of a Stylus contract.
#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("cargo error: {0}")]
    Cargo(#[from] escargot::error::CargoError),
    #[error("cargo metadata error: {0}")]
    CargoMetadata(#[from] cargo_metadata::Error),
    #[error("error fetching host: {0}")]
    RustcHost(#[from] rustc_host::Error),

    #[error("{0}")]
    Toolchain(#[from] crate::utils::toolchain::ToolchainError),

    #[error("failed to find shared library")]
    NoSharedLibraryFound,
    #[error("build did not generate wasm file")]
    NoWasmFound,
    #[error("more than one shared library found: {0} and {1}")]
    MultipleSharedLibraries(String, String),
}

/// Build a Stylus contract.
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

    // TODO: check output status
    let _status = cmd.into_command().status()?;

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

/// Build a native shared library for use in tracing.
pub fn build_shared_library(
    path: impl AsRef<Path>,
    package: Option<impl AsRef<OsStr>>,
    features: Option<Vec<String>>,
) -> Result<PathBuf, BuildError> {
    let mut cmd = Cargo::new().into_command();
    cmd.current_dir(&path)
        .args(["build", "--lib", "--locked"])
        .args(["--target", &sys::host_arch()?]);

    if let Some(features) = features {
        cmd.args(["--features", &features.join(",")]);
    }
    if let Some(package) = package {
        cmd.arg("--package").arg(package);
    }

    let _output = cmd.output()?;
    let shared_library = find_shared_library(&path)?;
    Ok(shared_library)
}

fn find_shared_library(project: impl AsRef<Path>) -> Result<PathBuf, BuildError> {
    // TODO: use metadata here
    let host_arch = sys::host_arch()?;
    let extension = sys::library_extension();
    let so_dir = project
        .as_ref()
        .join("target")
        .join(host_arch)
        .join("debug");
    let so_dir = fs::read_dir(so_dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file());

    let mut file: Option<PathBuf> = None;
    for entry in so_dir {
        let Some(ext) = entry.file_name() else {
            continue;
        };
        let ext = ext.to_string_lossy();

        if ext.contains(extension) {
            if let Some(other) = file {
                let other = other.file_name().unwrap().to_string_lossy().to_string();
                return Err(BuildError::MultipleSharedLibraries(ext.to_string(), other));
            }
            file = Some(entry);
        }
    }
    let Some(file) = file else {
        return Err(BuildError::NoSharedLibraryFound);
    };
    Ok(file)
}
