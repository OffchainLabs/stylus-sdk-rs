// Copyright 2025-2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Defines a struct that provides Stylus contracts access to a host VM
//! environment via the HostAccessor trait defined in stylus_host. Makes contracts
//! a lot more testable, as the VM can be mocked and injected upon initialization
//! of a storage type. Defines two implementations, one when the target arch is wasm32 and the
//! other when the target is native.

use alloc::vec::Vec;

use alloy_primitives::{B256, U256};
use stylus_core::*;

use crate::hostio;

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use stylus_core::calls::*;
        use stylus_core::calls::errors::*;
        use stylus_core::deploy::*;
        use crate::{call::{RawCall}};
        use alloy_primitives::{Address};
        use alloc::vec;

        /// Defines a struct that provides Stylus contracts access to a host VM
        /// environment via the HostAccessor trait defined in stylus_host.
        #[derive(Clone, Debug)]
        pub struct VM(pub WasmVM);

        impl Host for VM {}

        impl CryptographyAccess for VM {
            fn native_keccak256(&self, input: &[u8]) -> B256 {
                let mut output = B256::ZERO;
                unsafe {
                    hostio::native_keccak256(input.as_ptr(), input.len(), output.as_mut_ptr());
                }
                output
            }
        }

        impl CalldataAccess for VM {
            fn read_args(&self, len: usize) -> Vec<u8> {
                self.0.read_args(len)
            }
            fn read_return_data(&self, offset: usize, size: Option<usize>) -> Vec<u8> {
                self.0.read_return_data(offset, size)
            }
            fn return_data_size(&self) -> usize {
                self.0.return_data_size()
            }
            fn write_result(&self, data: &[u8]) {
                self.0.write_result(data)
            }
        }

        unsafe impl UnsafeDeploymentAccess for VM {
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

        impl StorageAccess for VM {
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

        unsafe impl UnsafeCallAccess for VM {
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

        macro_rules! unsafe_reentrant {
            ($block:block) => {
                #[cfg(feature = "reentrant")]
                unsafe {
                    $block
                }

                #[cfg(not(feature = "reentrant"))]
                $block
            };
        }

        impl CallAccess for VM {
            /// Calls the contract at the given address.
            fn call(
                &self,
                context: &dyn MutatingCallContext,
                to: alloy_primitives::Address,
                data: &[u8],
            ) -> Result<Vec<u8>, Error> {
                #[cfg(feature = "reentrant")]
                {
                    use stylus_core::host::StorageAccess;
                    self.flush_cache(true); // clear the storage to persist changes, invalidating the cache
                }

                unsafe_reentrant! {{
                    RawCall::<WasmVM>::new_with_value(context.value())
                        .gas(context.gas())
                        .call(to, data)
                        .map_err(Error::Revert)
                }}
            }
            /// Delegate calls the contract at the given address.
            ///
            /// # Safety
            ///
            /// A delegate call must trust the other contract to uphold safety requirements.
            /// Though this function clears any cached values, the other contract may arbitrarily change storage,
            /// spend ether, and do other things one should never blindly allow other contracts to do.
            unsafe fn delegate_call(
                &self,
                context: &dyn MutatingCallContext,
                to: alloy_primitives::Address,
                data: &[u8],
            ) -> Result<Vec<u8>, Error> {
                #[cfg(feature = "reentrant")]
                {
                    use stylus_core::host::StorageAccess;
                    self.flush_cache(true); // clear the storage to persist changes, invalidating the cache
                }

                RawCall::<WasmVM>::new_delegate()
                    .gas(context.gas())
                    .call(to, data)
                    .map_err(Error::Revert)
            }
            /// Static calls the contract at the given address.
            fn static_call(
                &self,
                context: &dyn StaticCallContext,
                to: alloy_primitives::Address,
                data: &[u8],
            ) -> Result<Vec<u8>, Error> {
                #[cfg(feature = "reentrant")]
                {
                    use stylus_core::host::StorageAccess;
                    self.flush_cache(false); // flush storage to persist changes, but don't invalidate the cache
                }

                unsafe_reentrant! {{
                    RawCall::<WasmVM>::new_static()
                        .gas(context.gas())
                        .call(to, data)
                        .map_err(Error::Revert)
                }}
            }
        }

        impl ValueTransfer for VM {
            /// Transfers an amount of ETH in wei to the given account.
            /// Note that this method will call the other contract, which may in turn call others.
            ///
            /// All gas is supplied, which the recipient may burn.
            /// If this is not desired, the [`call`] function may be used directly.
            ///
            /// [`call`]: super::call
            #[cfg(feature = "reentrant")]
            fn transfer_eth(
                &self,
                _storage: &mut dyn stylus_core::storage::TopLevelStorage,
                to: Address,
                amount: U256,
            ) -> Result<(), Vec<u8>> {
                use stylus_core::host::StorageAccess;
                self.flush_cache(true); // clear the storage to persist changes, invalidating the cache
                unsafe {
                    RawCall::<WasmVM>::new_with_value(amount)
                        .skip_return_data()
                        .call(to, &[])?;
                }
                Ok(())
            }
            /// Transfers an amount of ETH in wei to the given account.
            /// Note that this method will call the other contract, which may in turn call others.
            ///
            /// All gas is supplied, which the recipient may burn.
            /// If this is not desired, the [`call`] function may be used directly.
            ///
            /// ```
            /// # use stylus_sdk::call::{call, Call, transfer_eth};
            /// # fn wrap() -> Result<(), Vec<u8>> {
            /// #   let value = alloy_primitives::U256::ZERO;
            /// #   let recipient = alloy_primitives::Address::ZERO;
            /// transfer_eth(recipient, value)?;                 // these two are equivalent
            /// call(Call::new().value(value), recipient, &[])?; // these two are equivalent
            /// #     Ok(())
            /// # }
            /// ```
            #[cfg(not(feature = "reentrant"))]
            fn transfer_eth(&self, to: Address, amount: U256) -> Result<(), Vec<u8>> {
                RawCall::<WasmVM>::new_with_value(amount)
                    .skip_return_data()
                    .call(to, &[])?;
                Ok(())
            }
        }

        impl BlockAccess for VM {
            fn block_basefee(&self) -> U256 {
                let mut data = B256::ZERO;
                unsafe { hostio::block_basefee(data.as_mut_ptr()) };
                data.into()
            }
            fn block_coinbase(&self) -> Address {
                let mut data = Address::ZERO;
                unsafe { hostio::block_coinbase(data.as_mut_ptr()) };
                data.into()
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

        impl ChainAccess for VM {
            fn chain_id(&self) -> u64 {
                unsafe { hostio::chainid() }
            }
        }

        impl AccountAccess for VM {
            fn balance(&self, account: Address) -> U256 {
                let mut data = [0; 32];
                unsafe { hostio::account_balance(account.as_ptr(), data.as_mut_ptr()) };
                U256::from_be_bytes(data)
            }
            fn contract_address(&self) -> Address {
                let mut data = Address::ZERO;
                unsafe { hostio::contract_address(data.as_mut_ptr()) };
                data.into()
            }
            fn code(&self, account: Address) -> Vec<u8> {
                let size = unsafe { hostio::account_code_size(account.as_ptr()) };
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

        impl MemoryAccess for VM {
            fn pay_for_memory_grow(&self, pages: u16) {
                unsafe { hostio::pay_for_memory_grow(pages) }
            }
        }

        impl MessageAccess for VM {
            fn msg_reentrant(&self) -> bool {
                unsafe { hostio::msg_reentrant() }
            }
            fn msg_sender(&self) -> Address {
                let mut data = Address::ZERO;
                unsafe { hostio::msg_sender(data.as_mut_ptr()) };
                data.into()
            }
            fn msg_value(&self) -> U256 {
                let mut data = B256::ZERO;
                unsafe { hostio::msg_value(data.as_mut_ptr()) };
                data.into()
            }
            fn tx_origin(&self) -> Address {
                let mut data = Address::ZERO;
                unsafe { hostio::tx_origin(data.as_mut_ptr()) };
                data.into()
            }
        }

        impl MeteringAccess for VM {
            fn evm_gas_left(&self) -> u64 {
                self.0.evm_gas_left()
            }
            fn evm_ink_left(&self) -> u64 {
                self.0.evm_ink_left()
            }
            fn tx_gas_price(&self) -> U256 {
                self.0.tx_gas_price()
            }
            fn tx_ink_price(&self) -> u32 {
                self.0.tx_ink_price()
            }
        }
        macro_rules! unsafe_reentrant {
            ($block:block) => {
                #[cfg(feature = "reentrant")]
                unsafe {
                    $block
                }

                #[cfg(not(feature = "reentrant"))]
                $block
            };
        }
        impl DeploymentAccess for VM {
            #[cfg(feature = "reentrant")]
            unsafe fn deploy(
                &self,
                code: &[u8],
                endowment: U256,
                salt: Option<B256>,
                cache_policy: stylus_core::deploy::CachePolicy,
            ) -> Result<Address, Vec<u8>> {
                use stylus_core::deploy::CachePolicy;
                use stylus_core::host::StorageAccess;
                match cache_policy {
                    CachePolicy::Clear => self.flush_cache(true),
                    CachePolicy::Flush => self.flush_cache(false),
                    CachePolicy::DoNothing => {}
                }

                let mut contract = Address::default();
                let mut revert_data_len: usize = 0;

                let endowment: B256 = endowment.into();
                if let Some(salt) = salt {
                    self.create2(
                        code.as_ptr(),
                        code.len(),
                        endowment.as_ptr(),
                        salt.as_ptr(),
                        contract.as_mut_ptr(),
                        &mut revert_data_len as *mut _,
                    );
                } else {
                    self.create1(
                        code.as_ptr(),
                        code.len(),
                        endowment.as_ptr(),
                        contract.as_mut_ptr(),
                        &mut revert_data_len as *mut _,
                    );
                }
                if contract.is_zero() {
                    return Err(self.read_return_data(0, None));
                }
                Ok(contract)
            }
            #[cfg(not(feature = "reentrant"))]
            unsafe fn deploy(
                &self,
                code: &[u8],
                endowment: U256,
                salt: Option<B256>,
            ) -> Result<Address, Vec<u8>> {
                let mut contract = Address::default();
                let mut revert_data_len: usize = 0;

                let endowment: B256 = endowment.into();
                if let Some(salt) = salt {
                    self.create2(
                        code.as_ptr(),
                        code.len(),
                        endowment.as_ptr(),
                        salt.as_ptr(),
                        contract.as_mut_ptr(),
                        &mut revert_data_len as *mut _,
                    );
                } else {
                    self.create1(
                        code.as_ptr(),
                        code.len(),
                        endowment.as_ptr(),
                        contract.as_mut_ptr(),
                        &mut revert_data_len as *mut _,
                    );
                }
                if contract.is_zero() {
                    return Err(self.read_return_data(0, None));
                }
                Ok(contract)
            }
        }
        impl LogAccess for VM {
            fn emit_log(&self, input: &[u8], num_topics: usize) {
                unsafe { hostio::emit_log(input.as_ptr(), input.len(), num_topics) }
            }
            fn raw_log(&self, topics: &[B256], data: &[u8]) -> Result<(), &'static str> {
                if topics.len() > 4 {
                    return Err("too many topics");
                }
                let mut bytes: Vec<u8> = vec![];
                bytes.extend(topics.iter().flat_map(|x| x.0.iter()));
                bytes.extend(data);
                unsafe { hostio::emit_log(bytes.as_ptr(), bytes.len(), topics.len()) }
                Ok(())
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
pub struct WasmVM;

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
        unsafe { hostio::return_data_size() }
    }
    fn write_result(&self, data: &[u8]) {
        unsafe {
            hostio::write_result(data.as_ptr(), data.len());
        }
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
