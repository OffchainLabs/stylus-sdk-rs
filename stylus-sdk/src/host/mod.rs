// Copyright 2025-2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use stylus_host::*;

cfg_if::cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        /// Defines a struct that provides Stylus contracts access to a host VM
        /// environment via the HostAccessor trait defined in stylus_host.
        pub struct VM {
            /// A reference-counted host object that provides access to the VM
            /// for use in non-native mode. Reference counting avoids the need for
            /// unsafe code, explicit lifetimes, and other complexities.
            pub host: rclite::Rc<alloc::boxed::Box<dyn Host>>,
        }

        impl Clone for VM {
            fn clone(&self) -> Self {
                Self {
                    host: rclite::Rc::clone(&self.host),
                }
            }
        }

        impl core::fmt::Debug for VM {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "VM")
            }
        }
    } else {
        use alloc::vec::Vec;
        use alloy_primitives::{Address, B256, U256};

        use crate::{block, contract, evm, hostio, msg, tx, types::AddressVM};

        #[derive(Clone, Debug)]
        pub struct VM;

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

        unsafe impl DeploymentAccess for VM {
            unsafe fn create1(
                &self,
                code: Address,
                endowment: U256,
                contract: &mut Address,
                revert_data_len: &mut usize,
            ) {
                let endowment: B256 = endowment.into();
                hostio::create1(
                    code.as_ptr(),
                    code.len(),
                    endowment.as_ptr(),
                    contract.as_mut_ptr(),
                    revert_data_len as *mut _,
                );
            }
            unsafe fn create2(
                &self,
                code: Address,
                endowment: U256,
                salt: B256,
                contract: &mut Address,
                revert_data_len: &mut usize,
            ) {
                let endowment: B256 = endowment.into();
                hostio::create2(
                    code.as_ptr(),
                    code.len(),
                    endowment.as_ptr(),
                    salt.as_ptr(),
                    contract.as_mut_ptr(),
                    revert_data_len as *mut _,
                );
            }
        }

        impl StorageAccess for VM {
            fn emit_log(&self, input: &[u8], num_topics: usize) {
                unsafe { hostio::emit_log(input.as_ptr(), input.len(), num_topics) }
            }
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

        unsafe impl CallAccess for VM {
            unsafe fn call_contract(
                &self,
                to: Address,
                data: &[u8],
                value: U256,
                gas: u64,
                outs_len: &mut usize,
            ) -> u8 {
                let value = B256::from(value);
                hostio::call_contract(
                    to.as_ptr(),
                    data.as_ptr(),
                    data.len(),
                    value.as_ptr(),
                    gas,
                    outs_len,
                )
            }
            unsafe fn delegate_call_contract(
                &self,
                to: Address,
                data: &[u8],
                gas: u64,
                outs_len: &mut usize,
            ) -> u8 {
                hostio::delegate_call_contract(to.as_ptr(), data.as_ptr(), data.len(), gas, outs_len)
            }
            unsafe fn static_call_contract(
                &self,
                to: Address,
                data: &[u8],
                gas: u64,
                outs_len: &mut usize,
            ) -> u8 {
                hostio::static_call_contract(to.as_ptr(), data.as_ptr(), data.len(), gas, outs_len)
            }
        }

        impl BlockAccess for VM {
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

        impl ChainAccess for VM {
            fn chain_id(&self) -> u64 {
                block::chainid()
            }
        }

        impl AccountAccess for VM {
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

        impl MemoryAccess for VM {
            fn pay_for_memory_grow(&self, pages: u16) {
                evm::pay_for_memory_grow(pages)
            }
        }

        impl MessageAccess for VM {
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

        impl MeteringAccess for VM {
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
    }
}
