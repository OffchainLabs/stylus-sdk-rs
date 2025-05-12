// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::{io, path::PathBuf};

use serde::Deserialize;

use super::{
    build::{BuildConfig, CompilerArtifact, JsonMessage},
    cmd, manifest,
};
use crate::error::Result;

#[derive(Debug, Deserialize)]
pub struct Metadata {
    packages: Vec<Package>,
    target_directory: PathBuf,
    workspace_members: Vec<String>,
    workspace_default_members: Vec<String>,
}

impl Metadata {
    fn find_package_by_id(&self, id: &str) -> Option<&Package> {
        self.packages.iter().find(|package| package.id == id)
    }

    pub fn workspace_members(&self) -> impl Iterator<Item = &Package> {
        self.workspace_members
            .iter()
            .map(|id| self.find_package_by_id(id).unwrap())
    }

    pub fn workspace_default_members(&self) -> impl Iterator<Item = &Package> {
        self.workspace_default_members
            .iter()
            .map(|id| self.find_package_by_id(id).unwrap())
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Package {
    pub id: String,
    pub name: String,
    pub version: String,
    pub manifest_path: PathBuf,
}

impl Package {
    pub fn read_manifest(&self) -> Result<manifest::TomlManifest> {
        let manifest = toml::from_str(&std::fs::read_to_string(&self.manifest_path)?)?;
        Ok(manifest)
    }

    pub fn write_manifest(&self, manifest: &manifest::TomlManifest) -> Result<()> {
        std::fs::write(&self.manifest_path, &toml::to_string_pretty(manifest)?)?;
        Ok(())
    }

    pub fn build(&self, config: &BuildConfig) -> Result<CompilerArtifact> {
        // TODO: better error
        cmd::build(&self.manifest_path, config)?
            .into_iter()
            .filter_map(JsonMessage::as_compiler_artifact)
            .find(|artifact| artifact.package_id == self.id)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "compiler artifact for package not found",
                )
                .into()
            })
    }
}
