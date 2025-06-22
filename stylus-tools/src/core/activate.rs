// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Contract acitvation.
//!
//! See the [Arbitrum Docs](https://docs.arbitrum.io/stylus/concepts/how-it-works#activation) for
//! details on contract activation.

use alloy::{
    primitives::{utils::parse_ether, Address, Bytes, U256},
    providers::{Provider, WalletProvider},
    rpc::types::{
        state::{AccountOverride, StateOverride},
        TransactionReceipt,
    },
};

use crate::{precompiles, utils::bump_data_fee};

/// Activates an already deployed Stylus contract by address.
pub async fn contract(
    address: Address,
    data_fee_bump_percent: u64,
    provider: &(impl Provider + WalletProvider),
) -> Result<TransactionReceipt, ActivationError> {
    let code = provider.get_code_at(address).await?;
    let from_address = provider.default_signer_address();
    let data_fee = data_fee(code, address, &provider).await?;
    let data_fee = bump_data_fee(data_fee, data_fee_bump_percent);

    let receipt = precompiles::arb_wasm(&provider)
        .activateProgram(address)
        .from(from_address)
        .value(data_fee)
        .send()
        .await?
        .get_receipt()
        .await?;

    Ok(receipt)
}

/// Checks Stylus contract activation, returning the data fee.
pub async fn data_fee(
    code: Bytes,
    address: Address,
    provider: impl Provider,
) -> Result<U256, ActivationError> {
    let arbwasm = precompiles::arb_wasm(provider);
    let random_sender_addr = Address::random();
    let spoofed_sender_account = AccountOverride::default().with_balance(U256::MAX);
    let spoofed_code = AccountOverride::default().with_code(code.clone());
    let state_override = StateOverride::from_iter([
        (address, spoofed_code),
        (random_sender_addr, spoofed_sender_account),
    ]);

    let result = arbwasm
        .activateProgram(address)
        .state(state_override)
        .from(random_sender_addr)
        .value(parse_ether("1").unwrap())
        .call()
        .await?;

    Ok(result.dataFee)
}

/// Estimate gas cost for Stylus contract activation.
pub async fn estimate_gas(
    address: Address,
    data_fee_bump_percent: u64,
    provider: &(impl Provider + WalletProvider),
) -> Result<u64, ActivationError> {
    let code = provider.get_code_at(address).await?;
    let from_address = provider.default_signer_address();
    let data_fee = data_fee(code, address, &provider).await?;
    let data_fee = bump_data_fee(data_fee, data_fee_bump_percent);

    let gas = precompiles::arb_wasm(&provider)
        .activateProgram(address)
        .from(from_address)
        .value(data_fee)
        .estimate_gas()
        .await?;

    Ok(gas)
}

#[derive(Debug, thiserror::Error)]
pub enum ActivationError {
    #[error("{0}")]
    Contract(alloy::contract::Error),
    #[error("{0}")]
    PendingTransaction(#[from] alloy::providers::PendingTransactionError),
    #[error("{0}")]
    Rpc(#[from] alloy::transports::RpcError<alloy::transports::TransportErrorKind>),

    #[error("{0}")]
    MissingEntrypoint(MissingEntrypoint),
}

impl From<alloy::contract::Error> for ActivationError {
    fn from(err: alloy::contract::Error) -> Self {
        if err.to_string().contains("pay_for_memory_grow") {
            Self::MissingEntrypoint(MissingEntrypoint)
        } else {
            Self::Contract(err)
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error(
    "Contract could not be activated as it is missing an entrypoint. \
     Please ensure that your contract has an #[entrypoint] defined on your main struct"
)]
pub struct MissingEntrypoint;
