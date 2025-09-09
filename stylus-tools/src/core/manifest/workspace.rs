// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct WorkspaceManifest {
    pub workspace: TomlWorkspace,
}

#[derive(Debug, Deserialize)]
pub struct TomlWorkspace {
    pub networks: HashMap<String, TomlNetwork>,
}

#[derive(Debug, Deserialize)]
pub struct TomlNetwork {
    pub endpoint: String,
}
