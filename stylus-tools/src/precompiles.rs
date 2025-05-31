// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Precompile contracts on Arbitrum.

// TODO: better constructor API?
// TODO: docs on precompiles
// TODO: wrapper types

use alloy::{network::Network, providers::Provider, sol};

#[rustfmt::skip]
pub mod addresses {
    use alloy::primitives::{address, Address};

    pub const ARB_WASM: Address       = address!("0x0000000000000000000000000000000000000071");
    pub const ARB_WASM_CACHE: Address = address!("0x0000000000000000000000000000000000000072");
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
