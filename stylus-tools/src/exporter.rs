// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use crate::call;
use derive_builder::Builder;
use eyre::{OptionExt, Result};

/// Defines the configuration for exporting a Stylus contract.
/// After setting the parameters, call `Exporter::export_abi` or `Exporter::export_constructor` to perform the export.
#[derive(Builder)]
#[builder(setter(into))]
pub struct Exporter {
    #[builder(default)]
    dir: Option<String>,
}

impl Exporter {
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
