// Copyright 2025-2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Defines a struct that provides Stylus contracts access to a host VM
//! environment via the HostAccessor trait defined in stylus_host. Makes contracts
//! a lot more testable, as the VM can be mocked and injected upon initialization
//! of a storage type. Defines two implementations, one when the target arch is wasm32 and the
//! other when the target is native.

use alloc::vec::Vec;
use alloy_primitives::{Address, B256, U256};

use stylus_core::*;

use crate::{
    block, contract,
    evm::{self},
    hostio, msg, tx,
    types::AddressVM,
};

/// Defines an implementation of traits for the VM struct that
/// provides access to cross-contract calls.
pub mod calls;

/// Defines an implementation of traits for the VM struct
/// that provide access to programmatic contract deployment.
pub mod deploy;

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use stylus_core::calls::*;
        use stylus_core::deploy::*;

        /// Defines a struct that provides Stylus contracts access to a host VM
        /// environment via the HostAccessor trait defined in stylus_host.
        #[derive(Clone, Debug)]
        pub struct VM(pub WasmVM);

        impl Host for VM {}

        impl CryptographyAccess for VM {
            #[inline]
            fn native_keccak256(&self, input: &[u8]) -> B256 {
                self.0.native_keccak256(input)
            }
        }

        impl CalldataAccess for VM {
            #[inline]
            fn read_args(&self, len: usize) -> Vec<u8> {
                self.0.read_args(len)
            }
            #[inline]
            fn read_return_data(&self, offset: usize, size: Option<usize>) -> Vec<u8> {
                self.0.read_return_data(offset, size)
            }
            #[inline]
            fn return_data_size(&self) -> usize {
                self.0.return_data_size()
            }
            #[inline]
            fn write_result(&self, data: &[u8]) {
                self.0.write_result(data)
            }
        }

        unsafe impl UnsafeDeploymentAccess for VM {
            #[inline]
            unsafe fn create1(
                &self,
                code: *const u8,
                code_len: usize,
                endowment: *const u8,
                contract: *mut u8,
                revert_data_len: *mut usize,
            ) {
                self.0.create1(code, code_len, endowment, contract, revert_data_len)
            }
            #[inline]
            unsafe fn create2(
                &self,
                code: *const u8,
                code_len: usize,
                endowment: *const u8,
                salt: *const u8,
                contract: *mut u8,
                revert_data_len: *mut usize,
            ) {
                self.0.create2(code, code_len, endowment, salt, contract, revert_data_len)
            }
        }

        impl StorageAccess for VM {
            #[inline]
            unsafe fn storage_cache_bytes32(&self, key: U256, value: B256) {
                self.0.storage_cache_bytes32(key, value)
            }
            #[inline]
            fn storage_load_bytes32(&self, key: U256) -> B256 {
                self.0.storage_load_bytes32(key)
            }
            #[inline]
            fn flush_cache(&self, clear: bool) {
                self.0.flush_cache(clear)
            }
        }

        unsafe impl UnsafeCallAccess for VM {
            #[inline]
            unsafe fn call_contract(
                &self,
                to: *const u8,
                data: *const u8,
                data_len: usize,
                value: *const u8,
                gas: u64,
                outs_len: &mut usize,
            ) -> u8 {
                self.0.call_contract(to, data, data_len, value, gas, outs_len)
            }
            #[inline]
            unsafe fn delegate_call_contract(
                &self,
                to: *const u8,
                data: *const u8,
                data_len: usize,
                gas: u64,
                outs_len: &mut usize,
            ) -> u8 {
                self.0.delegate_call_contract(to, data, data_len, gas, outs_len)
            }
            #[inline]
            unsafe fn static_call_contract(
                &self,
                to: *const u8,
                data: *const u8,
                data_len: usize,
                gas: u64,
                outs_len: &mut usize,
            ) -> u8 {
                self.0.static_call_contract(to, data, data_len, gas, outs_len)
            }
        }

        impl BlockAccess for VM {
            #[inline]
            fn block_basefee(&self) -> U256 {
                self.0.block_basefee()
            }
            #[inline]
            fn block_coinbase(&self) -> Address {
                self.0.block_coinbase()
            }
            #[inline]
            fn block_gas_limit(&self) -> u64 {
                self.0.block_gas_limit()
            }
            #[inline]
            fn block_number(&self) -> u64 {
                self.0.block_number()
            }
            #[inline]
            fn block_timestamp(&self) -> u64 {
                self.0.block_timestamp()
            }
        }

        impl ChainAccess for VM {
            #[inline]
            fn chain_id(&self) -> u64 {
                self.0.chain_id()
            }
        }

        impl AccountAccess for VM {
            #[inline]
            fn balance(&self, account: Address) -> U256 {
                self.0.balance(account)
            }
            #[inline]
            fn contract_address(&self) -> Address {
                self.0.contract_address()
            }
            #[inline]
            fn code(&self, account: Address) -> Vec<u8> {
                self.0.code(account)
            }
            #[inline]
            fn code_size(&self, account: Address) -> usize {
                self.0.code_size(account)
            }
            #[inline]
            fn code_hash(&self, account: Address) -> B256 {
                self.0.code_hash(account)
            }
        }

        impl MemoryAccess for VM {
            #[inline]
            fn pay_for_memory_grow(&self, pages: u16) {
                self.0.pay_for_memory_grow(pages)
            }
        }

        impl MessageAccess for VM {
            #[inline]
            fn msg_reentrant(&self) -> bool {
                self.0.msg_reentrant()
            }
            #[inline]
            fn msg_sender(&self) -> Address {
                self.0.msg_sender()
            }
            #[inline]
            fn msg_value(&self) -> U256 {
                self.0.msg_value()
            }
            #[inline]
            fn tx_origin(&self) -> Address {
                self.0.tx_origin()
            }
        }

        impl MeteringAccess for VM {
            #[inline]
            fn evm_gas_left(&self) -> u64 {
                self.0.evm_gas_left()
            }
            #[inline]
            fn evm_ink_left(&self) -> u64 {
                self.0.evm_ink_left()
            }
            #[inline]
            fn tx_gas_price(&self) -> U256 {
                self.0.tx_gas_price()
            }
            #[inline]
            fn tx_ink_price(&self) -> u32 {
                self.0.tx_ink_price()
            }
        }
        impl CallAccess for VM {
            #[inline]
            fn call(
                &self,
                context: &dyn MutatingCallContext,
                to: alloy_primitives::Address,
                data: &[u8],
            ) -> Result<Vec<u8>, stylus_core::calls::errors::Error> {
                self.0.call(context, to, data)
            }
            #[inline]
            unsafe fn delegate_call(
                &self,
                context: &dyn MutatingCallContext,
                to: alloy_primitives::Address,
                data: &[u8],
            ) -> Result<Vec<u8>, stylus_core::calls::errors::Error> {
                self.0.delegate_call(context, to, data)
            }
            #[inline]
            fn static_call(
                &self,
                context: &dyn StaticCallContext,
                to: alloy_primitives::Address,
                data: &[u8],
            ) -> Result<Vec<u8>, stylus_core::calls::errors::Error> {
                self.0.static_call(context, to, data)
            }
        }
        impl ValueTransfer for VM {
            #[inline]
            #[cfg(feature = "reentrant")]
            fn transfer_eth(
                &self,
                storage: &mut dyn stylus_core::storage::TopLevelStorage,
                to: Address,
                amount: U256,
            ) -> Result<(), Vec<u8>> {
                self.0.transfer_eth(storage, to, amount)
            }
            #[inline]
            #[cfg(not(feature = "reentrant"))]
            fn transfer_eth(
                &self,
                to: alloy_primitives::Address,
                amount: alloy_primitives::U256,
            ) -> Result<(), Vec<u8>> {
                self.0.transfer_eth(to, amount)
            }
        }
        impl DeploymentAccess for VM {
            #[inline]
            #[cfg(feature = "reentrant")]
            unsafe fn deploy(
                &self,
                code: &[u8],
                endowment: U256,
                salt: Option<B256>,
                cache_policy: CachePolicy,
            ) -> Result<Address, Vec<u8>> {
                self.0.deploy(code, endowment, salt, cache_policy)
            }
            #[inline]
            #[cfg(not(feature = "reentrant"))]
            unsafe fn deploy(
                &self,
                code: &[u8],
                endowment: U256,
                salt: Option<B256>,
            ) -> Result<Address, Vec<u8>> {
                self.0.deploy(code, endowment, salt)
            }
        }
        impl LogAccess for VM {
            #[inline]
            fn emit_log(&self, input: &[u8], num_topics: usize) {
                self.0.emit_log(input, num_topics)
            }
            #[inline]
            fn raw_log(&self, topics: &[B256], data: &[u8]) -> Result<(), &'static str> {
                self.0.raw_log(topics, data)
            }
    }
    } else {
        /// Defines a struct that provides Stylus contracts access to a host VM
        /// environment via the HostAccessor trait defined in stylus_host.
        pub struct VM {
            /// A host object that provides access to the VM for use in native mode.
            pub host: alloc::boxed::Box<dyn Host>,
        }

        impl Clone for VM {
            fn clone(&self) -> Self {
                Self {
                    host: self.host.clone(),
                }
            }
        }

        impl core::fmt::Debug for VM {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "VM")
            }
        }
    }
}

/// Defines a struct that provides Stylus contracts access to a host VM
/// environment via the HostAccessor trait defined in stylus_host.
#[derive(Clone, Debug, Default)]
pub struct WasmVM {}

impl Host for WasmVM {}

#[allow(deprecated)]
impl CryptographyAccess for WasmVM {
    fn native_keccak256(&self, input: &[u8]) -> B256 {
        let mut output = B256::ZERO;
        unsafe {
            hostio::native_keccak256(input.as_ptr(), input.len(), output.as_mut_ptr());
        }
        output
    }
}

#[allow(deprecated)]
impl CalldataAccess for WasmVM {
    fn read_args(&self, len: usize) -> Vec<u8> {
        let mut input = Vec::with_capacity(len);
        unsafe {
            hostio::read_args(input.as_mut_ptr());
            input.set_len(len);
        }
        input
    }
    fn read_return_data(&self, offset: usize, size: Option<usize>) -> Vec<u8> {
        let size = size.unwrap_or_else(|| self.return_data_size().saturating_sub(offset));

        let mut data = Vec::with_capacity(size);
        if size > 0 {
            unsafe {
                let bytes_written = hostio::read_return_data(data.as_mut_ptr(), offset, size);
                debug_assert!(bytes_written <= size);
                data.set_len(bytes_written);
            }
        };
        data
    }
    fn return_data_size(&self) -> usize {
        contract::return_data_len()
    }
    fn write_result(&self, data: &[u8]) {
        unsafe {
            hostio::write_result(data.as_ptr(), data.len());
        }
    }
}

#[allow(deprecated)]
unsafe impl UnsafeDeploymentAccess for WasmVM {
    unsafe fn create1(
        &self,
        code: *const u8,
        code_len: usize,
        endowment: *const u8,
        contract: *mut u8,
        revert_data_len: *mut usize,
    ) {
        hostio::create1(code, code_len, endowment, contract, revert_data_len);
    }
    unsafe fn create2(
        &self,
        code: *const u8,
        code_len: usize,
        endowment: *const u8,
        salt: *const u8,
        contract: *mut u8,
        revert_data_len: *mut usize,
    ) {
        hostio::create2(code, code_len, endowment, salt, contract, revert_data_len);
    }
}

#[allow(deprecated)]
impl StorageAccess for WasmVM {
    unsafe fn storage_cache_bytes32(&self, key: U256, value: B256) {
        hostio::storage_cache_bytes32(B256::from(key).as_ptr(), value.as_ptr());
    }
    fn storage_load_bytes32(&self, key: U256) -> B256 {
        let mut data = B256::ZERO;
        unsafe { hostio::storage_load_bytes32(B256::from(key).as_ptr(), data.as_mut_ptr()) };
        data
    }
    fn flush_cache(&self, clear: bool) {
        unsafe { hostio::storage_flush_cache(clear) }
    }
}

#[allow(deprecated)]
impl LogAccess for WasmVM {
    fn emit_log(&self, input: &[u8], num_topics: usize) {
        unsafe { hostio::emit_log(input.as_ptr(), input.len(), num_topics) }
    }
    fn raw_log(&self, topics: &[B256], data: &[u8]) -> Result<(), &'static str> {
        evm::raw_log(topics, data)
    }
}

#[allow(deprecated)]
unsafe impl UnsafeCallAccess for WasmVM {
    unsafe fn call_contract(
        &self,
        to: *const u8,
        data: *const u8,
        data_len: usize,
        value: *const u8,
        gas: u64,
        outs_len: &mut usize,
    ) -> u8 {
        hostio::call_contract(to, data, data_len, value, gas, outs_len)
    }
    unsafe fn delegate_call_contract(
        &self,
        to: *const u8,
        data: *const u8,
        data_len: usize,
        gas: u64,
        outs_len: &mut usize,
    ) -> u8 {
        hostio::delegate_call_contract(to, data, data_len, gas, outs_len)
    }
    unsafe fn static_call_contract(
        &self,
        to: *const u8,
        data: *const u8,
        data_len: usize,
        gas: u64,
        outs_len: &mut usize,
    ) -> u8 {
        hostio::static_call_contract(to, data, data_len, gas, outs_len)
    }
}

#[allow(deprecated)]
impl BlockAccess for WasmVM {
    fn block_basefee(&self) -> U256 {
        block::basefee()
    }
    fn block_coinbase(&self) -> Address {
        block::coinbase()
    }
    fn block_gas_limit(&self) -> u64 {
        block::gas_limit()
    }
    fn block_number(&self) -> u64 {
        block::number()
    }
    fn block_timestamp(&self) -> u64 {
        block::timestamp()
    }
}

#[allow(deprecated)]
impl ChainAccess for WasmVM {
    fn chain_id(&self) -> u64 {
        block::chainid()
    }
}

#[allow(deprecated)]
impl AccountAccess for WasmVM {
    fn balance(&self, account: Address) -> U256 {
        account.balance()
    }
    fn contract_address(&self) -> Address {
        contract::address()
    }
    fn code(&self, account: Address) -> Vec<u8> {
        account.code()
    }
    fn code_size(&self, account: Address) -> usize {
        account.code_size()
    }
    fn code_hash(&self, account: Address) -> B256 {
        account.code_hash()
    }
}

#[allow(deprecated)]
impl MemoryAccess for WasmVM {
    fn pay_for_memory_grow(&self, pages: u16) {
        evm::pay_for_memory_grow(pages)
    }
}

#[allow(deprecated)]
impl MessageAccess for WasmVM {
    fn msg_reentrant(&self) -> bool {
        msg::reentrant()
    }
    fn msg_sender(&self) -> Address {
        msg::sender()
    }
    fn msg_value(&self) -> U256 {
        msg::value()
    }
    fn tx_origin(&self) -> Address {
        tx::origin()
    }
}

#[allow(deprecated)]
impl MeteringAccess for WasmVM {
    fn evm_gas_left(&self) -> u64 {
        evm::gas_left()
    }
    fn evm_ink_left(&self) -> u64 {
        evm::ink_left()
    }
    fn tx_gas_price(&self) -> U256 {
        tx::gas_price()
    }
    fn tx_ink_price(&self) -> u32 {
        tx::ink_price()
    }
}
