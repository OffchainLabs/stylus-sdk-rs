// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![allow(clippy::too_many_arguments)]

//! Precompile contracts on Arbitrum.

// TODO: better constructor API?
// TODO: docs on precompiles
// TODO: wrapper types

use alloy::{network::Network, providers::Provider, sol};

#[rustfmt::skip]
pub mod addresses {
    use alloy::primitives::{address, Address};

    pub const ARB_ADDRESS_TABLE: Address = address!("0x0000000000000000000000000000000000000066");
    pub const ARB_AGGREGATOR:    Address = address!("0x000000000000000000000000000000000000006D");
    pub const ARB_DEBUG:         Address = address!("0x00000000000000000000000000000000000000FF");
    pub const ARB_GAS_INFO:      Address = address!("0x000000000000000000000000000000000000006C");
    pub const ARB_INFO:          Address = address!("0x0000000000000000000000000000000000000065");
    pub const ARB_OWNER:         Address = address!("0x0000000000000000000000000000000000000070");
    pub const ARB_OWNER_PUBLIC:  Address = address!("0x000000000000000000000000000000000000006B");
    pub const ARB_RETRYABLE_TX:  Address = address!("0x000000000000000000000000000000000000006E");
    pub const ARB_SYS:           Address = address!("0x0000000000000000000000000000000000000064");
    pub const ARB_WASM:          Address = address!("0x0000000000000000000000000000000000000071");
    pub const ARB_WASM_CACHE:    Address = address!("0x0000000000000000000000000000000000000072");
}

pub fn arb_wasm<P: Provider<N>, N: Network>(provider: P) -> ArbWasm::ArbWasmInstance<P, N> {
    ArbWasm::new(addresses::ARB_WASM, provider)
}

pub fn arb_wasm_cache<P: Provider<N>, N: Network>(
    provider: P,
) -> ArbWasmCache::ArbWasmCacheInstance<P, N> {
    ArbWasmCache::new(addresses::ARB_WASM_CACHE, provider)
}

sol!("precompiles/ArbAddressTable.sol");
sol!("precompiles/ArbAggregator.sol");
sol!("precompiles/ArbDebug.sol");
sol!("precompiles/ArbGasInfo.sol");
sol!("precompiles/ArbInfo.sol");
sol!("precompiles/ArbOwner.sol");
sol!("precompiles/ArbOwnerPublic.sol");
sol!("precompiles/ArbRetryableTx.sol");
sol!("precompiles/ArbSys.sol");
sol!("precompiles/ArbWasm.sol");
sol!("precompiles/ArbWasmCache.sol");
