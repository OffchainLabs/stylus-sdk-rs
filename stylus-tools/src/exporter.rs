// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use crate::call;
use eyre::{OptionExt, Result};

/// Defines the configuration for exporting a Stylus contract.
/// After setting the parameters, call `Exporter::export_abi` or `Exporter::export_constructor` to perform the export.
#[derive(Default)]
pub struct Exporter {
    dir: Option<String>,
}

impl Exporter {
    // Create the Exporter with default parameters.
    pub fn new() -> Self {
        Self { dir: None }
    }

    pub fn with_contract_dir(mut self, dir: String) -> Self {
        self.dir = Some(dir);
        self
    }

    // Export the ABI of the Stylus contract.
    pub fn export_abi(&self) -> Result<String> {
        let res = call(&self.dir, "export-abi", vec![]);
        Ok(res?
            .split_once("pragma solidity ^0.8.23;")
            .ok_or_eyre("failed to parse abi")?
            .1
            .trim()
            .to_owned())
    }

    pub fn export_constructor(&self) -> Result<String> {
        let res = call(&self.dir, "constructor", vec![]);
        Ok(res?.trim().to_owned())
    }
}
