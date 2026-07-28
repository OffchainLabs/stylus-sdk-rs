// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::path::Path;

use alloy::{primitives::Address, providers::Provider};
use cargo_metadata::MetadataCommand;

use crate::core::{
    check::{check_contract, CheckConfig},
    project::{
        contract::{Contract, ContractStatus},
        ProjectHash,
    },
};

pub async fn check_workspace(config: &CheckConfig, provider: &impl Provider) -> eyre::Result<()> {
    let metadata = MetadataCommand::new().exec()?;
    let packages = metadata.workspace_default_packages();
    for package in packages {
        let contract = Contract::try_from(package)?;
        check_contract(&contract, None, config, provider).await?;
    }
    Ok(())
}

pub async fn check_wasm_file(
    wasm_file: impl AsRef<Path>,
    project_hash: ProjectHash,
    contract_address: Option<Address>,
    config: &CheckConfig,
    provider: &impl Provider,
) -> eyre::Result<ContractStatus> {
    // A raw `--wasm-file` is checked as-is: there is no project to resolve a wasm-opt recipe from.
    let status = crate::core::check::check_wasm_file(
        wasm_file,
        project_hash,
        None,
        contract_address,
        config,
        provider,
    )
    .await?;
    Ok(status)
}
