// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Defines host environment methods a Stylus contract has access to.
use alloc::vec::Vec;
use alloy_primitives::{Address, FixedBytes, B256, U256};

mod wasm;

pub use wasm::WasmHost;

/// The host trait defines methods a contract can use to interact
/// with a host environment, such as the EVM. It is a composition
/// of traits with different access to host values and modifications.
pub trait Host:
    CryptographyAccess
    + CalldataAccess
    + DeploymentAccess
    + StorageAccess
    + CallAccess
    + BlockAccess
    + ChainAccess
    + AccountAccess
    + MemoryAccess
    + MessageAccess
    + MeteringAccess
{
}

pub trait HostAccess<H: Host> {
    fn get_host(&self) -> &H;
}

/// TODO
pub trait CryptographyAccess {
    /// TODO
    fn native_keccak256(&self, input: &[u8]) -> FixedBytes<32>;
}

/// TODO
pub trait CalldataAccess {
    /// TODO
    fn args(&self, len: usize) -> Vec<u8>;
    /// TODO
    fn read_return_data(&self, offset: usize, size: Option<usize>) -> Vec<u8>;
    /// TODO
    fn return_data_len(&self) -> usize;
    /// TODO
    fn output(&self, data: &[u8]);
}

/// TODO
pub trait DeploymentAccess {
    /// TODO
    fn create1(&self);
    /// TODO
    fn create2(&self);
}

/// TODO
pub trait StorageAccess {
    /// TODO
    fn emit_log(&self, input: &[u8]);
    /// TODO
    fn load(&self, key: U256) -> B256;
    /// TODO
    fn cache(&self, key: U256, value: B256);
    /// TODO
    fn flush_cache(&self, clear: bool);
}

/// TODO
pub trait CallAccess {
    /// TODO
    fn call_contract(&self);
    /// TODO
    fn static_call_contract(&self);
    /// TODO
    fn delegate_call_contract(&self);
}

/// TODO
pub trait BlockAccess {
    /// TODO
    fn block_basefee(&self) -> U256;
    /// TODO
    fn block_coinbase(&self) -> Address;
    /// TODO
    fn block_number(&self) -> u64;
    /// TODO
    fn block_timestamp(&self) -> u64;
    /// TODO
    fn block_gas_limit(&self) -> u64;
}

/// TODO
pub trait ChainAccess {
    /// TODO
    fn chain_id(&self) -> u64;
}

/// TODO
pub trait AccountAccess {
    /// TODO
    fn balance(&self, account: Address) -> U256;
    /// TODO
    fn contract_address(&self) -> Address;
    /// TODO
    fn code(&self, account: Address) -> Vec<u8>;
    /// TODO
    fn code_size(&self, account: Address) -> usize;
    /// TODO
    fn codehash(&self, account: Address) -> FixedBytes<32>;
}

/// TODO
pub trait MemoryAccess {
    /// TODO
    fn pay_for_memory_grow(&self, pages: u16);
}

/// TODO
pub trait MessageAccess {
    /// TODO
    fn msg_sender(&self) -> Address;
    /// TODO
    fn msg_reentrant(&self) -> bool;
    /// TODO
    fn msg_value(&self) -> U256;
    /// TODO
    fn tx_origin(&self) -> Address;
}

/// TODO
pub trait MeteringAccess {
    /// TODO
    fn evm_gas_left(&self) -> u64;
    /// TODO
    fn evm_ink_left(&self) -> u64;
    /// TODO
    fn tx_gas_price(&self) -> U256;
    /// TODO
    fn tx_ink_price(&self) -> u32;
}
