// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::process::Output;

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

#[derive(Debug, Deserialize, Serialize)]
pub struct ProcessOutput {
    pub process_name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub stdout: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub stderr: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
}

impl ProcessOutput {
    pub fn check(process_name: impl Into<String>, output: Output) -> Result<String> {
        let process_output = ProcessOutput {
            process_name: process_name.into(),
            stdout: String::from_utf8(output.stdout)?,
            stderr: String::from_utf8(output.stderr)?,
            exit_code: output.status.code(),
        };
        if output.status.success() {
            Ok(process_output.stdout)
        } else {
            Err(Error::CommandFailure(process_output))
        }
    }
}
