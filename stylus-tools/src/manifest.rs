// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Stylus.toml manifest definitions.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TomlManifest {
    pub contract: Option<TomlContract>,
    pub workspace: Option<TomlWorkspace>,
}

impl TomlManifest {
    pub fn write<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        std::fs::write(path, toml::to_string_pretty(self)?)?;
        Ok(())
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TomlContract {}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TomlWorkspace {}
