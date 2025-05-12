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

sol! {
    #[sol(rpc)]
    interface ArbWasm {
        function activateProgram(address program) external payable returns (uint16 version, uint256 dataFee);
        function stylusVersion() external view returns (uint16 version);
        function codehashVersion(bytes32 codehash) external view returns (uint16 version);
        function codehashKeepalive(bytes32 codehash) external payable;
        function codehashAsmSize(bytes32 codehash) external view returns (uint32 size);
        function programVersion(address program) external view returns (uint16 version);
        function programInitGas(address program) external view returns (uint64 gas, uint64 gasWhenCached);
        function programMemoryFootprint(address program) external view returns (uint16 footprint);
        function programTimeLeft(address program) external view returns (uint64 _secs);
        function inkPrice() external view returns (uint32 price);
        function maxStackDepth() external view returns (uint32 depth);
        function freePages() external view returns (uint16 pages);
        function pageGas() external view returns (uint16 gas);
        function pageRamp() external view returns (uint64 ramp);
        function pageLimit() external view returns (uint16 limit);
        function minInitGas() external view returns (uint64 gas, uint64 cached);
        function initCostScalar() external view returns (uint64 percent);
        function expiryDays() external view returns (uint16 _days);
        function keepaliveDays() external view returns (uint16 _days);
        function blockCacheSize() external view returns (uint16 count);

        event ProgramActivated(
            bytes32 indexed codehash,
            bytes32 moduleHash,
            address program,
            uint256 dataFee,
            uint16 version
        );
        event ProgramLifetimeExtended(bytes32 indexed codehash, uint256 dataFee);

        error ProgramNotWasm();
        error ProgramNotActivated();
        error ProgramNeedsUpgrade(uint16 version, uint16 stylusVersion);
        error ProgramExpired(uint64 ageInSeconds);
        error ProgramUpToDate();
        error ProgramKeepaliveTooSoon(uint64 ageInSeconds);
        error ProgramInsufficientValue(uint256 have, uint256 want);
    }

    #[sol(rpc)]
    interface ArbWasmCache {
        function isCacheManager(address manager) external view returns (bool);
        function allCacheManagers() external view returns (address[] memory managers);
        function cacheCodehash(bytes32 codehash) external;
        function cacheProgram(address addr) external;
        function evictCodehash(bytes32 codehash) external;
        function codehashIsCached(bytes32 codehash) external view returns (bool);

        event UpdateProgramCache(address indexed manager, bytes32 indexed codehash, bool cached);
    }

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
