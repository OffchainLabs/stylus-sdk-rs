// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::{future::Future, path::Path};

use alloy::{
    primitives::{Address, FixedBytes},
    providers::Provider,
};
use bytesize::ByteSize;
use cargo_metadata::MetadataCommand;

use crate::{
    core::{
        activate,
        build::Build,
        contract::{contract_exists, ContractStatus},
        project::ProjectHash,
    },
    utils::format_file_size,
    wasm::process_wasm_file,
};

/// Checks that a contract is valid and can be deployed onchain.
///
/// Returns whether the WASM is already up-to-date and activated onchain, and the data fee.
pub async fn check(
    data_fee_bump_percent: u64,
    provider: &impl Provider,
) -> eyre::Result<Vec<ContractStatus>> {
    let metadata = MetadataCommand::new().exec()?;
    let packages = metadata.workspace_default_packages();
    let mut statuses = Vec::with_capacity(packages.len());
    for package in packages {
        let wasm_file = Build::contract(package)?.exec()?;
        // TODO: project hash
        // TODO: contract address?
        let status = check_wasm_file(
            &wasm_file,
            ProjectHash::default(),
            None,
            data_fee_bump_percent,
            provider,
        )
        .await?;
        statuses.push(status);
    }
    Ok(statuses)
}

pub async fn check_wasm_file(
    wasm_file: impl AsRef<Path>,
    project_hash: ProjectHash,
    contract_address: Option<Address>,
    data_fee_bump_percent: u64,
    provider: &impl Provider,
) -> eyre::Result<ContractStatus> {
    debug!(@grey, "reading wasm file at {}", wasm_file.as_ref().to_string_lossy().lavender());
    let processed = process_wasm_file(wasm_file, project_hash)?;
    info!(@grey, "contract size: {}", format_file_size(ByteSize::b(processed.code.len() as u64), ByteSize::kib(16), ByteSize::kib(24)));
    debug!(@grey, "wasm size: {}", format_file_size(ByteSize::b(processed.wasm.len() as u64), ByteSize::kib(96), ByteSize::kib(128)));

    // Check if the contract already exists
    // TODO: check log
    debug!(@grey, "connecting to RPC: {:?}", provider.root());
    let codehash = processed.codehash();
    if contract_exists(codehash, &provider).await? {
        return Ok(ContractStatus::Active {
            code: processed.code,
        });
    }

    let contract_address = contract_address.unwrap_or_else(Address::random);
    let fee = activate::data_fee(
        processed.code.clone(),
        contract_address,
        data_fee_bump_percent,
        provider,
    )
    .await?;
    Ok(ContractStatus::Ready {
        code: processed.code,
        fee,
    })
}
