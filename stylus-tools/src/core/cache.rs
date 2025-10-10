// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::{
    primitives::{Address, Uint, U256},
    providers::{Provider, WalletProvider},
    sol,
};
use bytesize::ByteSize;
use log::log_enabled;

use crate::{
    error::{decode_contract_error, ContractDecodeError},
    precompiles,
    utils::color::{Color, DebugColor},
};

sol! {
    #[sol(rpc)]
    interface CacheManager {
        function cacheSize() external view returns (uint64);
        function queueSize() external view returns (uint64);
        function isPaused() external view returns (bool);
        function placeBid(address program) external payable;
        function getMinBid(address program) external view returns (uint192 min);
        function getMinBid(uint64 size) public view returns (uint192 min);

        error AsmTooLarge(uint256 asm, uint256 queueSize, uint256 cacheSize);
        error AlreadyCached(bytes32 codehash);
        error BidTooSmall(uint192 bid, uint192 min);
        error BidsArePaused();
        error ProgramNotActivated();
    }
}

pub async fn cache_manager<P: Provider>(
    provider: &P,
) -> Result<CacheManager::CacheManagerInstance<&P>, CacheError> {
    let address = cache_manager_address(provider).await?;
    let cache_manager = CacheManager::new(address, provider);
    Ok(cache_manager)
}

async fn cache_manager_address(provider: &impl Provider) -> Result<Address, CacheError> {
    let arb_wasm_cache = precompiles::arb_wasm_cache(provider);
    let mut managers = arb_wasm_cache.allCacheManagers().call().await?;
    managers.pop().ok_or(CacheError::NoCacheManagers)
}

/// Attempts to cache a Stylus contract by address by placing a bid by sending a tx to the network.
pub async fn place_bid(
    address: Address,
    bid: impl Into<U256>,
    max_fee_per_gas_wei: Option<u128>,
    provider: &(impl Provider + WalletProvider),
) -> Result<(), CacheError> {
    let from_address = provider.default_signer_address();
    let cache_manager = cache_manager(provider).await?;
    let mut place_bid_call = cache_manager.placeBid(address).value(bid.into());
    if let Some(max_fee) = max_fee_per_gas_wei {
        place_bid_call = place_bid_call.max_fee_per_gas(max_fee);
        place_bid_call = place_bid_call.max_priority_fee_per_gas(0);
    };

    info!(@grey, "Checking if contract can be cached...");

    place_bid_call
        .clone()
        .from(from_address)
        .call()
        .await
        .map_err(decode_contract_error::<CacheManager::CacheManagerErrors>)?;
    info!(@grey, "Sending cache bid tx...");
    let pending_tx = place_bid_call.send().await?;
    let receipt = pending_tx.get_receipt().await?;
    if log_enabled!(log::Level::Debug) {
        let gas = format_gas(receipt.gas_used.into());
        debug!(
            @grey,
            "Successfully cached contract at address: {address} {} {gas} gas used",
            "with".grey()
        );
    } else {
        info!(@grey, "Successfully cached contract at address: {address}");
    }
    let tx_hash = receipt.transaction_hash.debug_lavender();
    info!(@grey, "Sent Stylus cache bid tx with hash: {tx_hash}");
    Ok(())
}

pub async fn status<P: Provider>(
    address: Option<Address>,
    cache_manager: &CacheManager::CacheManagerInstance<P>,
    provider: P,
) -> Result<CacheManagerStatus, CacheError> {
    let is_paused = cache_manager.isPaused().call().await?;
    let queue_size = ByteSize::b(cache_manager.queueSize().call().await?);
    let cache_size = ByteSize::b(cache_manager.cacheSize().call().await?);
    let min_bid_8kb = cache_manager
        .getMinBid_1(ByteSize::kb(8).as_u64())
        .call()
        .await?;
    let min_bid_16kb = cache_manager
        .getMinBid_1(ByteSize::kb(16).as_u64())
        .call()
        .await?;
    let min_bid_24kb = cache_manager
        .getMinBid_1(ByteSize::kb(24).as_u64())
        .call()
        .await?;

    let is_cached = match address {
        Some(address) => {
            let arb_wasm_cache = precompiles::arb_wasm_cache(&provider);
            let code = provider.get_code_at(address).await?;
            let codehash = alloy::primitives::keccak256(code);
            arb_wasm_cache.codehashIsCached(codehash).call().await?
        }
        None => false,
    };

    Ok(CacheManagerStatus {
        is_paused,
        queue_size,
        cache_size,
        min_bid_8kb,
        min_bid_16kb,
        min_bid_24kb,
        is_cached,
    })
}

#[derive(Debug)]
pub struct CacheManagerStatus {
    pub is_paused: bool,
    pub queue_size: ByteSize,
    pub cache_size: ByteSize,
    pub min_bid_8kb: Uint<192, 3>,
    pub min_bid_16kb: Uint<192, 3>,
    pub min_bid_24kb: Uint<192, 3>,
    pub is_cached: bool,
}

/// Recommends a minimum bid to the user for caching a Stylus program by address.
pub async fn min_bid(
    address: Address,
    provider: &impl Provider,
) -> Result<Uint<192, 3>, CacheError> {
    let cache_manager = cache_manager(provider).await?;
    cache_manager
        .getMinBid_0(address)
        .call()
        .await
        .map_err(|e| decode_contract_error::<CacheManager::CacheManagerErrors>(e).into())
}

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("contract error: {0}")]
    Contract(#[from] alloy::contract::Error),
    #[error("pending transaction error: {0}")]
    PendingTransaction(#[from] alloy::providers::PendingTransactionError),
    #[error("rpc error: {0}")]
    Rpc(#[from] alloy::transports::RpcError<alloy::transports::TransportErrorKind>),

    #[error("{0}")]
    ContractDecode(#[from] crate::error::ContractDecodeError),

    #[error("no cache managers found in ArbWasmCache, perhaps the Stylus cache is not yet enabled on this chain")]
    NoCacheManagers,
    #[error("Stylus contract was too large to cache")]
    AsmTooLarge,
    #[error("Stylus contract is already cached")]
    AlreadyCached,
    #[error("Bidding is currently paused for the Stylus cache manager")]
    BidsArePaused,
    #[error("Bid amount (wei) too small")]
    BidTooSmall,
    #[error("Your Stylus contract is not yet activated. To activate it, use the `cargo stylus activate` subcommand")]
    ProgramNotActivated,
}

impl From<CacheManager::CacheManagerErrors> for CacheError {
    fn from(err: CacheManager::CacheManagerErrors) -> Self {
        match err {
            CacheManager::CacheManagerErrors::AsmTooLarge(_) => Self::AsmTooLarge,
            CacheManager::CacheManagerErrors::AlreadyCached(_) => Self::AlreadyCached,
            CacheManager::CacheManagerErrors::BidsArePaused(_) => Self::BidsArePaused,
            CacheManager::CacheManagerErrors::BidTooSmall(_) => Self::BidTooSmall,
            CacheManager::CacheManagerErrors::ProgramNotActivated(_) => Self::ProgramNotActivated,
        }
    }
}

impl From<Result<CacheManager::CacheManagerErrors, ContractDecodeError>> for CacheError {
    fn from(err: Result<CacheManager::CacheManagerErrors, ContractDecodeError>) -> Self {
        match err {
            Ok(err) => err.into(),
            Err(err) => err.into(),
        }
    }
}

pub fn format_gas(gas: u128) -> String {
    let text = format!("{gas} gas");
    if gas <= 3_000_000 {
        text.mint()
    } else if gas <= 7_000_000 {
        text.yellow()
    } else {
        text.pink()
    }
}
