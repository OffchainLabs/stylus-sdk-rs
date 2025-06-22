// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Contract acitvation.
//!
//! See the [Arbitrum Docs](https://docs.arbitrum.io/stylus/concepts/how-it-works#activation) for
//! more info on contract activation.

use alloy::{
    primitives::{utils::format_units, Address},
    providers::{Provider, WalletProvider},
};

use crate::{core::activate, utils::color::DebugColor};

pub async fn activate_contract(
    address: Address,
    data_fee_bump_percent: u64,
    provider: &(impl Provider + WalletProvider),
) -> eyre::Result<()> {
    let receipt = activate::contract(address, data_fee_bump_percent, provider).await?;
    greyln!(
        "successfully activated contract 0x{} with tx {}",
        hex::encode(address),
        hex::encode(receipt.transaction_hash).debug_lavender()
    );
    Ok(())
}

pub async fn activation_estimate_gas(
    address: Address,
    data_fee_bump_percent: u64,
    provider: &(impl Provider + WalletProvider),
) -> eyre::Result<()> {
    let gas = activate::estimate_gas(address, data_fee_bump_percent, provider).await?;
    let gas_price = provider.get_gas_price().await?;

    greyln!("estimates");
    greyln!("activation tx gas: {}", gas.debug_lavender());
    greyln!(
        "gas price: {} gwei",
        format_units(gas_price, "gwei")?.debug_lavender()
    );

    let total_cost = gas_price.checked_mul(gas.into()).unwrap_or_default();
    let eth_estimate = format_units(total_cost, "ether")?;

    greyln!(
        "activation tx total cost: {} ETH",
        eth_estimate.debug_lavender()
    );

    Ok(())
}
