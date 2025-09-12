// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::collections::HashMap;

use alloy::primitives::Address;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ContractManifest {
    pub contract: TomlContract,
}

#[derive(Debug, Deserialize)]
pub struct TomlContract {
    pub deployments: HashMap<String, TomlDeployment>,
}

#[derive(Debug, Deserialize)]
pub struct TomlDeployment {
    pub network: String,
    pub no_activate: bool,
    pub deployer_address: Address,
}
