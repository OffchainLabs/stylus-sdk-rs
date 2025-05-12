// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::{
    primitives::{Address, U256},
    providers::{Provider, WalletProvider},
};

use crate::{
    core::cache::{self, cache_manager},
    utils::color::DebugColor,
};

/// Attempts to cache a Stylus contract by address by placing a bid by sending a tx to the network.
///
/// It will handle the different cache manager errors that can be encountered along the way and
/// print friendlier errors if failed.
pub async fn place_bid(
    address: Address,
    bid: impl Into<U256>,
    max_fee_per_gas_wei: Option<u128>,
    provider: &(impl Provider + WalletProvider),
) -> eyre::Result<()> {
    cache::place_bid(address, bid, max_fee_per_gas_wei, provider).await?;
    Ok(())
}

/// Checks the status of the Stylus cache manager, including the cache size, queue size, and
/// minimum bid for different contract sizes as reference points.
///
/// It also checks if a specified Stylus contract address is currently cached.
pub async fn status(address: Option<Address>, provider: &impl Provider) -> eyre::Result<()> {
    let cache_manager = cache_manager(provider).await?;
    let status = cache::status(address, &cache_manager, provider).await?;

    greyln!(
        "Cache manager address: {}",
        cache_manager.address().debug_lavender()
    );
    greyln!(
        "Cache manager status: {}",
        if status.is_paused {
            "paused".debug_red()
        } else {
            "active".debug_mint()
        }
    );
    greyln!("Cache size: {}", status.cache_size.debug_grey());
    greyln!("Queue size: {}", status.queue_size.debug_grey());
    greyln!(
        "Minimum bid for {} contract: {}",
        "8kb".debug_mint(),
        status.min_bid_8kb.debug_lavender()
    );
    greyln!(
        "Minimum bid for {} contract: {}",
        "16kb".debug_yellow(),
        status.min_bid_16kb.debug_lavender()
    );
    greyln!(
        "Minimum bid for {} contract: {}",
        "24kb".debug_red(),
        status.min_bid_24kb.debug_lavender()
    );
    if status.queue_size < status.cache_size {
        greyln!("Cache is not yet at capacity, so bids of size 0 are accepted");
    } else {
        greyln!("Cache is at capacity, bids must be >= 0 to be accepted");
    }

    if let Some(address) = address {
        greyln!(
            "Contract at address {} {}",
            address.debug_lavender(),
            if status.is_cached {
                "is cached".debug_mint()
            } else {
                "is not yet cached".debug_red() + " please use cargo stylus cache bid to cache it"
            }
        );
    }

    Ok(())
}

/// Recommends a minimum bid to the user for caching a Stylus program by address.
///
/// If the program has not yet been activated, the user will be informed.
pub async fn suggest_bid(address: Address, provider: &impl Provider) -> eyre::Result<()> {
    let min_bid = cache::min_bid(address, provider).await?;
    greyln!(
        "Minimum bid for contract {address}: {} wei",
        min_bid.debug_mint()
    );
    Ok(())
}
