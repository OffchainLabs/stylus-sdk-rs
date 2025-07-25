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

use crate::{
    core::activation::{self, ActivationConfig},
    utils::color::DebugColor,
};

pub async fn contract(
    address: Address,
    config: &ActivationConfig,
    provider: &(impl Provider + WalletProvider),
) -> eyre::Result<()> {
    let receipt = activation::activate_contract(address, config, provider).await?;
    info!(@grey,
        "successfully activated contract 0x{} with tx {}",
        hex::encode(address),
        hex::encode(receipt.transaction_hash).debug_lavender()
    );
    Ok(())
}

pub async fn estimate_gas(
    address: Address,
    config: &ActivationConfig,
    provider: &(impl Provider + WalletProvider),
) -> eyre::Result<()> {
    let gas = activation::estimate_gas(address, config, provider).await?;
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
