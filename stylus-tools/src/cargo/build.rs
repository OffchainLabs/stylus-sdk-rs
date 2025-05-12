// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Cargo build utilities.

use serde::Deserialize;

#[derive(Debug, Default)]
pub struct BuildConfig {
    pub opt_level: OptLevel,
    pub stable: bool,
    pub features: Option<String>,
}

#[derive(Debug, Default)]
pub enum OptLevel {
    #[default]
    S,
    Z,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "reason")]
pub enum JsonMessage {
    BuildFinished(BuildFinished),
    CompilerArtifact(CompilerArtifact),
    CompilerMessage(CompilerMessage),
}

impl JsonMessage {
    pub fn as_compiler_artifact(self) -> Option<CompilerArtifact> {
        match self {
            Self::CompilerArtifact(msg) => Some(msg),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BuildFinished {
    pub success: bool,
}

#[derive(Debug, Deserialize)]
pub struct CompilerArtifact {
    pub package_id: String,
    pub filenames: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CompilerMessage {}
