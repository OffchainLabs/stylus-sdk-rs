// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::path::Path;

use alloy::{primitives::Address, providers::Provider};
use bytesize::ByteSize;

use crate::{
    core::{
        activation::{self, ActivationConfig},
        build::{build_contract, BuildConfig},
        project::{
            contract::{Contract, ContractStatus},
            hash_project, ProjectConfig, ProjectHash,
        },
    },
    utils::format_file_size,
    wasm::process_wasm_file,
};

#[derive(Debug, Default)]
pub struct CheckConfig {
    pub activation: ActivationConfig,
    pub build: BuildConfig,
    pub project: ProjectConfig,
}

#[derive(Debug, thiserror::Error)]
pub enum CheckError {
    #[error("cargo metadata error: {0}")]
    CargoMetadata(#[from] cargo_metadata::Error),

    #[error("{0}")]
    Activation(#[from] crate::core::activation::ActivationError),
    #[error("{0}")]
    Build(#[from] crate::core::build::BuildError),
    #[error("{0}")]
    Contract(#[from] crate::core::project::contract::ContractError),
    #[error("{0}")]
    Project(#[from] crate::core::project::ProjectError),
    #[error("{0}")]
    ProcessWasm(#[from] crate::wasm::ProcessWasmFileError),
}

/// Checks that a contract is valid and can be deployed onchain.
///
/// Returns whether the WASM is already up-to-date and activated onchain, and the data fee.
pub async fn check_contract(
    contract: &Contract,
    config: &CheckConfig,
    provider: &impl Provider,
) -> Result<ContractStatus, CheckError> {
    let wasm_file = build_contract(contract, &config.build)?;
    let project_hash = hash_project(&config.project)?;
    let status = check_wasm_file(&wasm_file, project_hash, None, config, provider).await?;
    Ok(status)
}

pub async fn check_wasm_file(
    wasm_file: impl AsRef<Path>,
    project_hash: ProjectHash,
    contract_address: Option<Address>,
    config: &CheckConfig,
    provider: &impl Provider,
) -> Result<ContractStatus, CheckError> {
    debug!(@grey, "reading wasm file at {}", wasm_file.as_ref().to_string_lossy().lavender());
    let processed = process_wasm_file(wasm_file, project_hash)?;
    info!(@grey, "contract size: {}", format_file_size(ByteSize::b(processed.code.len() as u64), ByteSize::kib(16), ByteSize::kib(24)));
    debug!(@grey, "wasm size: {}", format_file_size(ByteSize::b(processed.wasm.len() as u64), ByteSize::kib(96), ByteSize::kib(128)));

    // Check if the contract already exists
    // TODO: check log
    debug!(@grey, "connecting to RPC: {:?}", provider.root());
    let codehash = processed.codehash();
    if Contract::exists(codehash, &provider).await? {
        return Ok(ContractStatus::Active {
            code: processed.code,
        });
    }

    let contract_address = contract_address.unwrap_or_else(Address::random);
    let fee = activation::data_fee(
        processed.code.clone(),
        contract_address,
        &config.activation,
        provider,
    )
    .await?;
    Ok(ContractStatus::Ready {
        code: processed.code,
        fee,
    })
}
