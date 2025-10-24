// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Types for JSON formatted output of Docker CLI commands.

use serde::Deserialize;

/// Output of each image in `docker images`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct Image {
    pub id: Option<String>,
    pub repository: String,
    pub created_at: String,
    pub tag: String,
}
