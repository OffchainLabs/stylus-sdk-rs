// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Defines a testing framework for Stylus contracts. Allows for easy unit testing
//! of programs by using a MockHost which implements the [`Host`] trait and provides
//! Foundry-style cheatcodes for programs.

use alloc::vec::Vec;
use alloy_primitives::{Address, FixedBytes, B256, U256};

use crate::host::*;
/// The host trait defines methods a contract can use to interact
/// with a host environment, such as the EVM.

/// Extends the host trait to implement Foundry-style cheatcodes.
pub trait CheatcodeProvider: Host {
    /// Set block.timestamp
    fn warp(&mut self, t: U256);

    /// Set block.number
    fn roll(&mut self, block_num: U256);

    /// Set block.basefee
    fn fee(&mut self, fee: U256);

    /// Set block.difficulty
    /// Does not work from the Paris hard fork and onwards, and will revert instead.
    fn difficulty(&mut self, diff: U256);

    /// Set block.prevrandao
    /// Does not work before the Paris hard fork, and will revert instead.
    fn prevrandao(&mut self, randao: FixedBytes<32>);

    /// Set block.chainid
    fn chain_id(&mut self, id: U256);

    /// Loads a storage slot from an address
    fn load(&self, account: Address, slot: FixedBytes<32>) -> FixedBytes<32>;

    /// Stores a value to an address' storage slot
    fn store(&mut self, account: Address, slot: FixedBytes<32>, value: FixedBytes<32>);

    /// Sets the *next* call's msg.sender to be the input address
    fn prank(&mut self, sender: Address);
}

#[derive(Default)]
pub struct MockHost;

impl Host for MockHost {}

impl CryptographyAccess for MockHost {
    fn native_keccak256(&self, input: &[u8]) -> FixedBytes<32> {
        FixedBytes::<32>::default()
    }
}

impl CalldataAccess for MockHost {
    fn args(&self, len: usize) -> Vec<u8> {
        Vec::new()
    }
    fn read_return_data(&self, offset: usize, size: Option<usize>) -> Vec<u8> {
        Vec::new()
    }
    fn return_data_len(&self) -> usize {
        0
    }
    fn output(&self, data: &[u8]) {}
}

impl DeploymentAccess for MockHost {
    fn create1(&self) {}
    fn create2(&self) {}
}

impl StorageAccess for MockHost {
    fn emit_log(&self, input: &[u8]) {}
    fn load(&self, key: U256) -> B256 {
        B256::default()
    }
    fn cache(&self, key: U256, value: B256) {}
    fn flush_cache(&self, clear: bool) {}
}

impl CallAccess for MockHost {
    fn call_contract(&self) {}
    fn static_call_contract(&self) {}
    fn delegate_call_contract(&self) {}
}

impl BlockAccess for MockHost {
    fn block_basefee(&self) -> U256 {
        U256::ZERO
    }
    fn block_coinbase(&self) -> Address {
        Address::ZERO
    }
    fn block_number(&self) -> u64 {
        0
    }
    fn block_timestamp(&self) -> u64 {
        0
    }
    fn block_gas_limit(&self) -> u64 {
        0
    }
}

impl ChainAccess for MockHost {
    fn chain_id(&self) -> u64 {
        0
    }
}

impl AccountAccess for MockHost {
    fn balance(&self, account: Address) -> U256 {
        U256::ZERO
    }
    fn contract_address(&self) -> Address {
        Address::ZERO
    }
    fn code(&self, account: Address) -> Vec<u8> {
        Vec::new()
    }
    fn code_size(&self, account: Address) -> usize {
        0
    }
    fn codehash(&self, account: Address) -> FixedBytes<32> {
        FixedBytes::<32>::default()
    }
}

impl MemoryAccess for MockHost {
    fn pay_for_memory_grow(&self, pages: u16) {}
}

impl MessageAccess for MockHost {
    fn msg_sender(&self) -> Address {
        Address::ZERO
    }
    fn msg_reentrant(&self) -> bool {
        false
    }
    fn msg_value(&self) -> U256 {
        U256::ZERO
    }
    fn tx_origin(&self) -> Address {
        Address::ZERO
    }
}

impl MeteringAccess for MockHost {
    fn evm_gas_left(&self) -> u64 {
        0
    }
    fn evm_ink_left(&self) -> u64 {
        0
    }
    fn tx_gas_price(&self) -> U256 {
        U256::ZERO
    }
    fn tx_ink_price(&self) -> u32 {
        0
    }
}

//
//impl CheatcodeProvider for MockHost {
//    fn warp(&mut self, t: U256) {}
//    fn roll(&mut self, block_num: U256) {}
//    fn fee(&mut self, fee: U256) {}
//    fn difficulty(&mut self, diff: U256) {}
//    fn prevrandao(&mut self, randao: FixedBytes<32>) {}
//    fn chain_id(&mut self, id: U256) {}
//    fn load(&self, account: Address, slot: FixedBytes<32>) -> FixedBytes<32> {
//        slot
//    }
//    fn store(&mut self, account: Address, slot: FixedBytes<32>, value: FixedBytes<32>) {}
//    fn prank(&mut self, sender: Address) {}
//}
