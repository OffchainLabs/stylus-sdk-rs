// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::{
    io,
    path::{Path, PathBuf},
};

use bytesize::ByteSize;

use crate::{
    cargo::{self, build::BuildConfig},
    error::Result,
    utils::format_file_size,
    wasm::{process_wasm_file, ProcessedWasm},
};

/// A Stylus contract
#[derive(Debug)]
pub struct Contract {
    path: PathBuf,
    cargo_package: cargo::metadata::Package,
}

impl Contract {
    /// Create reference to an existing Stylus contract.
    pub fn new<P: Into<PathBuf>>(path: P, cargo_package: cargo::metadata::Package) -> Self {
        let path = path.into();
        Self {
            path,
            cargo_package,
        }
    }

    /// Create a new stylus contract.
    ///
    /// This function uses `cargo new` to create a rust crate, then initializes it as a Stylus
    /// contract.
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        cargo::cmd::new(path)?;

        // TODO: is this the right way to get this?
        let cargo_metadata = cargo::cmd::metadata(path)?;
        let cargo_workspace_default_members = cargo_metadata
            .workspace_default_members()
            .collect::<Vec<_>>();
        debug_assert_eq!(cargo_workspace_default_members.len(), 1);
        let cargo_package = cargo_workspace_default_members[0].clone();

        // read generated Cargo.toml manifest
        let mut cargo_manifest = cargo_package.read_manifest()?;
        // read Cargo.toml template
        let cargo_template: cargo::manifest::TomlManifest =
            toml::from_str(include_str!("../../templates/contract/Cargo.toml"))?;
        // merge template into Cargo.toml, keeping original [package] section
        cargo_manifest.dependencies = cargo_template.dependencies;
        cargo_manifest.dev_dependencies = cargo_template.dev_dependencies;
        cargo_manifest.features = cargo_template.features;
        cargo_manifest.bin = cargo_template.bin;
        cargo_manifest.lib = cargo_template.lib;
        cargo_manifest.profile = cargo_template.profile;
        // write edited Cargo.toml
        cargo_package.write_manifest(&cargo_manifest)?;

        copy_from_template!(
            "../../templates/contract" -> path,
            "src/lib.rs",
            "src/main.rs",
            "rust-toolchain.toml",
            "Stylus.toml",
        );
        Ok(Self {
            cargo_package,
            path: path.into(),
        })
    }

    // TODO: move this into standalone function?
    // TODO: this has nothing stylus specific?
    /// Build a contract to WASM and return a path to the compiled WASM file.
    pub fn build_dylib(&self, config: &BuildConfig) -> Result<PathBuf> {
        // Enforce a version is included in the Cargo.toml file.
        let cargo_toml_version = &self.cargo_package.version;
        info!(@grey, "Building project with Cargo.toml version: {cargo_toml_version}");

        // TODO: extract this into sanitize() utility?
        // TODO: why are we removing quotes
        let project_name = self.cargo_package.name.replace('-', "_").replace('"', "");
        let expected_name = format!("{project_name}.wasm");

        // TODO: better error
        let wasm_file_path = self
            .cargo_package
            .build(config)?
            .filenames
            .into_iter()
            .find(|filename| filename == &expected_name)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "could not find WASM in release dir",
                )
            })?;

        /*
        let ProcessedWasm { wasm, code } =
            process_wasm_file(&wasm_file_path, [0; 32]).map_err(|err| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("failed to compress Wasm: {err}"),
                )
            })?;

        // TODO: constants
        info!(
            @grey, "contract size: {}",
            format_file_size(ByteSize::b(code.len() as u64), ByteSize::kib(16), ByteSize::kib(24))
        );
        info!(
            @grey, "wasm size: {}",
            format_file_size(ByteSize::b(wasm.len() as u64), ByteSize::kib(96), ByteSize::kib(128))
        );
        */
        Ok(wasm_file_path.into())
    }

    /// Check validity of a Stylus contract.
    pub async fn check(&self) {
        // TODO: check stylus testnet
    }
}

impl TryFrom<cargo::metadata::Package> for Contract {
    type Error = crate::error::Error;

    fn try_from(value: cargo::metadata::Package) -> Result<Self, Self::Error> {
        // TODO: is unwrap safe here?
        // TODO: formal definition for location of Cargo.toml
        let path = value.manifest_path.parent().unwrap().to_path_buf();
        path.join("Stylus.toml")
            .exists()
            .then(|| Self::new(path, value))
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "missing Stylus.toml").into())
    }
}
