// Copyright 2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::{
    primitives::Address,
    providers::{Provider, WalletProvider},
};

use super::{request::DeploymentRequest, DeploymentConfig, DeploymentError};
use crate::core::code::{contract::ContractCode, fragments::CodeFragments};

/// Deploy contract fragments, and return the root contract code
pub async fn deploy_fragments(
    fragments: &CodeFragments,
    uncompressed_code_size: u32,
    config: &DeploymentConfig,
    provider: &(impl Provider + WalletProvider),
) -> Result<ContractCode, DeploymentError> {
    let from_address = provider.default_signer_address();
    let mut addresses = Vec::new();
    for fragment in fragments.as_slice() {
        let req =
            DeploymentRequest::new(from_address, fragment.bytes(), config.max_fee_per_gas_gwei);
        let receipt = req.exec(&provider).await?;
        let address = receipt
            .contract_address
            .ok_or(DeploymentError::MissingReceiptAddress)?;
        addresses.push(address);
    }
    Ok(ContractCode::new_root_contract(
        uncompressed_code_size,
        addresses,
    ))
}

/// Estimate the gas for deploying code fragments
///
/// This does not include the cost of deploying the root contract or activation.
pub async fn estimate_fragments_gas(
    fragments: &CodeFragments,
    config: &DeploymentConfig,
    provider: &(impl Provider + WalletProvider),
) -> Result<u64, DeploymentError> {
    let from_address = provider.default_signer_address();
    let mut gas = 0;
    for fragment in fragments.as_slice() {
        let req =
            DeploymentRequest::new(from_address, fragment.bytes(), config.max_fee_per_gas_gwei);
        gas += req.estimate_gas(provider).await?;
    }
    Ok(gas)
}

/// Estimate the gas for deploying the root contract
pub async fn estimate_root_contract_gas(
    fragments: &CodeFragments,
    uncompressed_code_size: u32,
    config: &DeploymentConfig,
    provider: &(impl Provider + WalletProvider),
) -> Result<u64, DeploymentError> {
    let from_address = provider.default_signer_address();
    let root = ContractCode::new_root_contract(
        uncompressed_code_size,
        fragments.as_slice().iter().map(|_| Address::ZERO),
    );
    let req = DeploymentRequest::new(from_address, root.bytes(), config.max_fee_per_gas_gwei);
    req.estimate_gas(provider).await
}
