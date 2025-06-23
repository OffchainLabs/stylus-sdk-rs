// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
};

use cargo_metadata::{semver::Version, Package, TargetKind};
use escargot::{format::Message, CargoBuild};

use crate::{
    utils::{cargo::parse_messages_for_filename, toolchain::get_toolchain_channel},
    Result,
};

const WASM_TARGET: &str = "wasm32-unknown-unknown";
const OPT_LEVEL_Z_CONFIG: &str = "profile.release.opt-level='z'";
const UNSTABLE_FLAGS: &[&str] = &[
    "build-std=std,panic_abort",
    "build-std-features=panic_immediate_abort",
];

#[derive(Debug)]
pub struct Build {
    // Build flags
    opt_level: OptLevel,
    stable: bool,
    features: Vec<String>,

    // Cargo.toml metadata
    name: String,
    version: Version,
}

impl Build {
    pub fn contract(package: &Package) -> Result<Self> {
        let toolchain_channel = get_toolchain_channel(package)?;
        let stable = !toolchain_channel.contains("nightly");
        let version = package.version.clone();
        // First, let's try to find if the library's name is set, since this will interfere with
        // finding the wasm file in the deps directory if it's different.
        let name = package
            .targets
            .iter()
            .find_map(|t| t.kind.contains(&TargetKind::Lib).then(|| t.name.clone()))
            // If that doesn't work, then we can use the package name, and break normally.
            .unwrap_or_else(|| package.name.to_string());
        Ok(Self {
            stable,
            version,
            name,
            opt_level: OptLevel::default(),
            features: Vec::new(),
        })
    }

    pub fn features(&mut self, features: impl IntoIterator<Item = impl Into<String>>) -> &mut Self {
        self.features.extend(features.into_iter().map(Into::into));
        self
    }

    pub fn exec(&self) -> Result<PathBuf, BuildError> {
        info!(@grey, "Building project with Cargo.toml version: {}", self.version);
        let mut cmd = CargoBuild::new()
            .args(["--lib", "--locked"])
            .release()
            .target(WASM_TARGET);

        if !self.features.is_empty() {
            cmd = cmd.features(self.features.join(" "));
        }

        if !self.stable {
            cmd = cmd.args(UNSTABLE_FLAGS.iter().flat_map(|flag| ["-Z", flag]));
        }

        if matches!(self.opt_level, OptLevel::Z) {
            cmd = cmd.args(["--config", OPT_LEVEL_Z_CONFIG]);
        }

        let messages = cmd.exec()?;
        let path = parse_messages_for_filename(messages, format!("{}.wasm", self.name))?
            .ok_or(BuildError::NoWasmFound)?;
        Ok(path)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("cargo error: {0}")]
    Cargo(#[from] escargot::error::CargoError),

    #[error("build did not generate wasm file")]
    NoWasmFound,
}

#[derive(Debug, Default)]
pub enum OptLevel {
    #[default]
    S,
    Z,
}
