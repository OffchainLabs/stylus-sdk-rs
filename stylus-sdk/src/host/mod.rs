// Copyright 2025-2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Defines a struct that provides Stylus contracts access to a host VM
//! environment via the [`HostAccess`] trait defined in `stylus_core`. Makes contracts
//! a lot more testable, as the VM can be mocked and injected upon initialization
//! of a storage type. Defines two implementations, one when the stylus-test feature
//! is enabled and another that calls the actual HostIOs.
//!
//! # Allocator safety
//!
//! Several host I/O helpers use [`Vec::set_len`] on freshly-allocated buffers
//! whose contents are then filled by the host. This is sound **only** when the
//! allocator hands back zeroed memory (avoiding reads of uninitialised bytes).
//! The default `mini-alloc` crate satisfies this because it is a bump allocator
//! over fresh WebAssembly pages, which the Wasm spec guarantees are zero-filled.
//! Using a different allocator that returns uninitialised memory may introduce
//! undefined behaviour. If you disable the `mini-alloc` feature to supply your
//! own allocator, ensure it zeroes newly-allocated pages or audit every
//! `set_len` call site in this module.

use alloc::vec::Vec;

use alloy_primitives::{Address, B256, U256};
use stylus_core::*;

use crate::hostio;

cfg_if::cfg_if! {
    if #[cfg(not(feature = "stylus-test"))] {
        /// Defines a struct that provides Stylus contracts access to a host VM
        /// environment via the [`HostAccess`] trait defined in `stylus_core`.
        pub struct VM {
            /// A WebAssembly host that provides access to the VM onchain.
            pub host: WasmVM,
        }
    } else {
        /// Defines a struct that provides Stylus contracts access to a host VM
        /// environment via the [`HostAccess`] trait defined in `stylus_core`.
        pub struct VM {
            /// A host object that provides access to the VM for use in native mode.
            pub host: alloc::boxed::Box<dyn Host>,
        }
    }
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

impl Host for VM {}

impl CryptographyAccess for VM {
    #[inline]
    fn native_keccak256(&self, input: &[u8]) -> B256 {
        self.host.native_keccak256(input)
    }
}

impl CalldataAccess for VM {
    #[inline]
    fn read_args(&self, len: usize) -> Vec<u8> {
        self.host.read_args(len)
    }
    #[inline]
    fn read_return_data(&self, offset: usize, size: Option<usize>) -> Vec<u8> {
        self.host.read_return_data(offset, size)
    }
    #[inline]
    fn return_data_size(&self) -> usize {
        self.host.return_data_size()
    }
    #[inline]
    fn write_result(&self, data: &[u8]) {
        self.host.write_result(data)
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
        self.host
            .create1(code, code_len, endowment, contract, revert_data_len)
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
        self.host
            .create2(code, code_len, endowment, salt, contract, revert_data_len)
    }
}

impl StorageAccess for VM {
    #[inline]
    unsafe fn storage_cache_bytes32(&self, key: U256, value: B256) {
        self.host.storage_cache_bytes32(key, value)
    }
    #[inline]
    fn storage_load_bytes32(&self, key: U256) -> B256 {
        self.host.storage_load_bytes32(key)
    }
    #[inline]
    fn flush_cache(&self, clear: bool) {
        self.host.flush_cache(clear)
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
        self.host
            .call_contract(to, data, data_len, value, gas, outs_len)
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
        self.host
            .delegate_call_contract(to, data, data_len, gas, outs_len)
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
        self.host
            .static_call_contract(to, data, data_len, gas, outs_len)
    }
}

impl BlockAccess for VM {
    #[inline]
    fn block_basefee(&self) -> U256 {
        self.host.block_basefee()
    }
    #[inline]
    fn block_coinbase(&self) -> Address {
        self.host.block_coinbase()
    }
    #[inline]
    fn block_gas_limit(&self) -> u64 {
        self.host.block_gas_limit()
    }
    #[inline]
    fn block_number(&self) -> u64 {
        self.host.block_number()
    }
    #[inline]
    fn block_timestamp(&self) -> u64 {
        self.host.block_timestamp()
    }
}

impl ChainAccess for VM {
    #[inline]
    fn chain_id(&self) -> u64 {
        self.host.chain_id()
    }
}

impl AccountAccess for VM {
    #[inline]
    fn balance(&self, account: Address) -> U256 {
        self.host.balance(account)
    }
    #[inline]
    fn contract_address(&self) -> Address {
        self.host.contract_address()
    }
    #[inline]
    fn code(&self, account: Address) -> Vec<u8> {
        self.host.code(account)
    }
    #[inline]
    fn code_size(&self, account: Address) -> usize {
        self.host.code_size(account)
    }
    #[inline]
    fn code_hash(&self, account: Address) -> B256 {
        self.host.code_hash(account)
    }
}

impl MemoryAccess for VM {
    #[inline]
    fn pay_for_memory_grow(&self, pages: u16) {
        self.host.pay_for_memory_grow(pages)
    }
}

impl MessageAccess for VM {
    #[inline]
    fn msg_reentrant(&self) -> bool {
        self.host.msg_reentrant()
    }
    #[inline]
    fn msg_sender(&self) -> Address {
        self.host.msg_sender()
    }
    #[inline]
    fn msg_value(&self) -> U256 {
        self.host.msg_value()
    }
    #[inline]
    fn tx_origin(&self) -> Address {
        self.host.tx_origin()
    }
}

impl MeteringAccess for VM {
    #[inline]
    fn evm_gas_left(&self) -> u64 {
        self.host.evm_gas_left()
    }
    #[inline]
    fn evm_ink_left(&self) -> u64 {
        self.host.evm_ink_left()
    }
    #[inline]
    fn tx_gas_price(&self) -> U256 {
        self.host.tx_gas_price()
    }
    #[inline]
    fn tx_ink_price(&self) -> u32 {
        self.host.tx_ink_price()
    }
}

impl RawLogAccess for VM {
    #[inline]
    fn emit_log(&self, input: &[u8], num_topics: usize) {
        self.host.emit_log(input, num_topics)
    }
    #[inline]
    fn raw_log(&self, topics: &[B256], data: &[u8]) -> Result<(), &'static str> {
        self.host.raw_log(topics, data)
    }
}

/// WebAssembly host VM implementation that delegates to on-chain host I/O
/// functions. This is the default host used by deployed contracts running
/// on-chain. When the `stylus-test` feature is enabled, a mock host is
/// used instead for native unit testing.
#[derive(Clone, Debug, Default)]
pub struct WasmVM {}

impl Host for WasmVM {}

impl CryptographyAccess for WasmVM {
    fn native_keccak256(&self, input: &[u8]) -> B256 {
        let mut output = B256::ZERO;
        unsafe {
            hostio::native_keccak256(input.as_ptr(), input.len(), output.as_mut_ptr());
        }
        output
    }
}

impl CalldataAccess for WasmVM {
    fn read_args(&self, len: usize) -> Vec<u8> {
        // SAFETY: `set_len(len)` on uninitialized memory is sound here because
        // `hostio::read_args` writes exactly `len` bytes (the host always writes
        // the full calldata), and the default allocator (`mini-alloc`) is a bump
        // allocator over fresh WASM pages which are guaranteed zeroed by the
        // WASM spec — so even bytes beyond the calldata are zero, not uninit.
        let mut input = Vec::with_capacity(len);
        unsafe {
            hostio::read_args(input.as_mut_ptr());
            input.set_len(len);
        }
        input
    }
    fn read_return_data(&self, offset: usize, size: Option<usize>) -> Vec<u8> {
        let size = size.unwrap_or_else(|| self.return_data_size().saturating_sub(offset));

        // SAFETY: `set_len(bytes_written)` is sound because `bytes_written <= size`
        // (the host respects the requested size), and `mini-alloc` is a bump
        // allocator over fresh WASM pages (zeroed by spec) so the capacity
        // region is not uninit garbage.
        let mut data = Vec::with_capacity(size);
        if size > 0 {
            unsafe {
                let bytes_written = hostio::read_return_data(data.as_mut_ptr(), offset, size);
                debug_assert!(bytes_written <= size);
                data.set_len(bytes_written);
            }
        }
        data
    }
    fn return_data_size(&self) -> usize {
        unsafe { hostio::return_data_size() }
    }
    fn write_result(&self, data: &[u8]) {
        unsafe {
            hostio::write_result(data.as_ptr(), data.len());
        }
    }
}

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

impl RawLogAccess for WasmVM {
    fn emit_log(&self, input: &[u8], num_topics: usize) {
        unsafe { hostio::emit_log(input.as_ptr(), input.len(), num_topics) }
    }
    fn raw_log(&self, topics: &[B256], data: &[u8]) -> Result<(), &'static str> {
        if topics.len() > 4 {
            return Err("too many topics");
        }
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend(topics.iter().flat_map(|x| x.0.iter()));
        bytes.extend(data);
        self.emit_log(&bytes, topics.len());
        Ok(())
    }
}

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

impl BlockAccess for WasmVM {
    fn block_basefee(&self) -> U256 {
        unsafe {
            let mut data = B256::ZERO;
            hostio::block_basefee(data.as_mut_ptr());
            data.into()
        }
    }
    fn block_coinbase(&self) -> Address {
        unsafe {
            let mut data = Address::ZERO;
            hostio::block_coinbase(data.as_mut_ptr());
            data
        }
    }
    fn block_gas_limit(&self) -> u64 {
        unsafe { hostio::block_gas_limit() }
    }
    fn block_number(&self) -> u64 {
        unsafe { hostio::block_number() }
    }
    fn block_timestamp(&self) -> u64 {
        unsafe { hostio::block_timestamp() }
    }
}

impl ChainAccess for WasmVM {
    fn chain_id(&self) -> u64 {
        unsafe { hostio::chainid() }
    }
}

impl AccountAccess for WasmVM {
    fn balance(&self, account: Address) -> U256 {
        let mut data = [0; 32];
        unsafe { hostio::account_balance(account.as_ptr(), data.as_mut_ptr()) };
        U256::from_be_bytes(data)
    }
    fn contract_address(&self) -> Address {
        let mut data = Address::ZERO;
        unsafe { hostio::contract_address(data.as_mut_ptr()) };
        data
    }
    fn code(&self, account: Address) -> Vec<u8> {
        let size = self.code_size(account);
        // SAFETY: `set_len(size)` is sound because `account_code` writes
        // exactly `min(size, actual_code_len)` bytes, and code is immutable
        // within a transaction so `size` from `code_size` is accurate.
        // `mini-alloc` is a bump allocator over fresh WASM pages (zeroed by
        // spec), so capacity bytes are not uninit garbage.
        let mut dest = Vec::with_capacity(size);
        unsafe {
            hostio::account_code(account.as_ptr(), 0, size, dest.as_mut_ptr());
            dest.set_len(size);
            dest
        }
    }
    fn code_size(&self, account: Address) -> usize {
        unsafe { hostio::account_code_size(account.as_ptr()) }
    }
    fn code_hash(&self, account: Address) -> B256 {
        let mut data = [0; 32];
        unsafe { hostio::account_codehash(account.as_ptr(), data.as_mut_ptr()) };
        data.into()
    }
}

impl MemoryAccess for WasmVM {
    fn pay_for_memory_grow(&self, pages: u16) {
        unsafe { hostio::pay_for_memory_grow(pages) }
    }
}

impl MessageAccess for WasmVM {
    fn msg_reentrant(&self) -> bool {
        unsafe { hostio::msg_reentrant() }
    }
    fn msg_sender(&self) -> Address {
        let mut data = Address::ZERO;
        unsafe { hostio::msg_sender(data.as_mut_ptr()) };
        data
    }
    fn msg_value(&self) -> U256 {
        let mut data = B256::ZERO;
        unsafe { hostio::msg_value(data.as_mut_ptr()) };
        data.into()
    }
    fn tx_origin(&self) -> Address {
        let mut data = Address::ZERO;
        unsafe { hostio::tx_origin(data.as_mut_ptr()) };
        data
    }
}

impl MeteringAccess for WasmVM {
    fn evm_gas_left(&self) -> u64 {
        unsafe { hostio::evm_gas_left() }
    }
    fn evm_ink_left(&self) -> u64 {
        unsafe { hostio::evm_ink_left() }
    }
    fn tx_gas_price(&self) -> U256 {
        let mut data = B256::ZERO;
        unsafe { hostio::tx_gas_price(data.as_mut_ptr()) };
        data.into()
    }
    fn tx_ink_price(&self) -> u32 {
        unsafe { hostio::tx_ink_price() }
    }
}

/// Provides a way to access the VM struct directly.
pub trait VMAccess {
    /// Returns a copy of the VM.
    ///
    /// # Safety
    ///
    /// This is unsafe because it might cause aliasing with existing slots defined by the contract.
    unsafe fn raw_vm(&self) -> VM;
}

#[cfg(test)]
mod tests {
    // These tests exercise the `CalldataAccess` / `AccountAccess` trait contracts
    // via `TestVM`. The `WasmVM` implementation delegates to real host I/O
    // functions and cannot be tested natively.
    use stylus_test::vm::TestVM;

    use super::*;

    #[test]
    fn test_read_return_data_returns_correct_slice() {
        let vm = TestVM::new();
        let data = vec![0xaa, 0xbb, 0xcc, 0xdd, 0xee];
        vm.write_result(&data);

        let result = vm.read_return_data(0, None);
        assert_eq!(result, data);
    }

    #[test]
    fn test_read_return_data_with_offset() {
        let vm = TestVM::new();
        let data = vec![0xaa, 0xbb, 0xcc, 0xdd, 0xee];
        vm.write_result(&data);

        let result = vm.read_return_data(2, None);
        assert_eq!(result, vec![0xcc, 0xdd, 0xee]);
    }

    #[test]
    fn test_read_return_data_with_size() {
        let vm = TestVM::new();
        let data = vec![0xaa, 0xbb, 0xcc, 0xdd, 0xee];
        vm.write_result(&data);

        let result = vm.read_return_data(1, Some(2));
        assert_eq!(result, vec![0xbb, 0xcc]);
    }

    #[test]
    fn test_read_return_data_truncates_when_size_exceeds_available() {
        let vm = TestVM::new();
        let data = vec![0xaa, 0xbb];
        vm.write_result(&data);

        let result = vm.read_return_data(0, Some(100));
        assert_eq!(result, data);
    }

    #[test]
    fn test_read_return_data_empty_when_offset_beyond_data() {
        let vm = TestVM::new();
        let data = vec![0xaa, 0xbb];
        vm.write_result(&data);

        let result = vm.read_return_data(10, None);
        assert!(result.is_empty());
    }

    #[test]
    fn test_read_return_data_empty_data() {
        let vm = TestVM::new();
        let result = vm.read_return_data(0, None);
        assert!(result.is_empty());
    }

    #[test]
    fn test_read_return_data_offset_plus_size_exceeds_available() {
        let vm = TestVM::new();
        let data = vec![0xaa, 0xbb, 0xcc, 0xdd, 0xee];
        vm.write_result(&data);

        let result = vm.read_return_data(3, Some(10));
        assert_eq!(result, vec![0xdd, 0xee]);
    }

    #[test]
    fn test_write_result_overwrites_previous_data() {
        let vm = TestVM::new();
        vm.write_result(&[0xaa, 0xbb]);
        vm.write_result(&[0xcc]);

        assert_eq!(vm.return_data_size(), 1);
        let result = vm.read_return_data(0, None);
        assert_eq!(result, vec![0xcc]);
    }

    #[test]
    fn test_read_return_data_explicit_zero_size() {
        let vm = TestVM::new();
        vm.write_result(&[0xaa, 0xbb, 0xcc]);

        let result = vm.read_return_data(0, Some(0));
        assert!(result.is_empty());
    }

    #[test]
    fn test_code_returns_stored_code() {
        let vm = TestVM::new();
        let addr = Address::with_last_byte(0x42);
        let code = vec![0x60, 0x00, 0x60, 0x00, 0xfd];
        vm.set_code(addr, code.clone());

        let result = vm.code(addr);
        assert_eq!(result, code);
    }

    #[test]
    fn test_code_returns_empty_for_unknown_address() {
        let vm = TestVM::new();
        let addr = Address::with_last_byte(0x99);

        let result = vm.code(addr);
        assert!(result.is_empty());
    }

    #[test]
    fn test_code_size_matches_code_length() {
        let vm = TestVM::new();
        let addr = Address::with_last_byte(0x42);
        let code = vec![0x60, 0x00, 0x60, 0x00, 0xfd];
        vm.set_code(addr, code.clone());

        assert_eq!(vm.code_size(addr), code.len());
        assert_eq!(vm.code_size(Address::ZERO), 0);
    }
}
