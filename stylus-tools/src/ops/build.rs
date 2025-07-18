// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::{path::PathBuf, str::FromStr};

use cargo_metadata::MetadataCommand;
use cargo_util_schemas::manifest::PackageName;
use eyre::eyre;

use crate::core::{
    build::{self, BuildConfig},
    contract::Contract,
};

/// Build contracts in a workspace.
pub fn build_workspace(config: &BuildConfig) -> eyre::Result<Vec<eyre::Result<PathBuf>>> {
    let metadata = MetadataCommand::new().exec()?;
    Ok(metadata
        .workspace_default_packages()
        .into_iter()
        .map(|package| {
            let contract = Contract::try_from(package)?;
            let wasm_path = build::build_contract(&contract, config)?;
            Ok(wasm_path)
        })
        .collect())
}

/// Build a Stylus contract to WASM and return the path to the compiled WASM file.
pub fn build_contract(
    package_name: impl AsRef<str>,
    config: &BuildConfig,
) -> eyre::Result<PathBuf> {
    let package_name = PackageName::from_str(package_name.as_ref())?;

    let metadata = MetadataCommand::new().exec()?;
    let package = metadata
        .packages
        .into_iter()
        .find(|p| p.name == package_name)
        .ok_or(eyre!("could not find contract: {package_name}"))?;
    let contract = Contract::try_from(&package)?;

    let wasm_path = build::build_contract(&contract, config)?;
    Ok(wasm_path)
}
