use crate::{
    block, call::Call, contract, evm, host::*, hostio, msg, storage::Storage, tx, types::AddressVM,
};

/// WasmHost is the default implementation of the host trait
/// for all Stylus programs using the SDK.
pub struct WasmHost;
// impl Host for WasmHost {}

// impl CryptographyAccess for WasmHost {
//     fn native_keccak256(&self, input: &[u8]) -> FixedBytes<32> {
//         FixedBytes::<32>::default()
//     }
// }

impl CalldataAccess for WasmHost {
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

impl StorageAccess for WasmHost {
    fn emit_log(&self, input: &[u8], num_topics: usize) {
        unsafe { hostio::emit_log(input.as_ptr(), input.len(), num_topics) }
    }
    // TODO: Make this an unsafe func?
    fn storage_cache_bytes32(&self, key: U256, value: B256) {
        unsafe {
            hostio::storage_cache_bytes32(B256::from(key).as_ptr(), value.as_ptr());
        }
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

impl BlockAccess for WasmHost {
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

// impl DeploymentAccess for WasmHost {
//     fn create1(&self) {}
//     fn create2(&self) {}
// }

// impl StorageAccess for WasmHost {
//     fn emit_log(&self, input: &[u8]) {}
//     fn load(&self, key: U256) -> B256 {
//         let mut data = B256::ZERO;
//         unsafe { hostio::storage_load_bytes32(B256::from(key).as_ptr(), data.as_mut_ptr()) };
//         data
//     }
//     fn cache(&self, key: U256, value: B256) {
//         // TODO: This converts the global storage method from unsafe to safe. Should we do this?
//         unsafe { hostio::storage_cache_bytes32(B256::from(key).as_ptr(), value.as_ptr()) }
//     }
//     fn flush_cache(&self, clear: bool) {
//         unsafe {
//             hostio::storage_flush_cache(clear);
//         }
//     }
// }

// impl CallAccess for WasmHost {
//     fn call_contract(&self) {}
//     fn static_call_contract(&self) {}
//     fn delegate_call_contract(&self) {}
// }

// impl BlockAccess for WasmHost {
//     fn block_basefee(&self) -> U256 {
//         block::basefee()
//     }
//     fn block_coinbase(&self) -> Address {
//         block::coinbase()
//     }
//     fn block_number(&self) -> u64 {
//         block::number()
//     }
//     fn block_timestamp(&self) -> u64 {
//         block::timestamp()
//     }
//     fn block_gas_limit(&self) -> u64 {
//         block::gas_limit()
//     }
// }

// impl ChainAccess for WasmHost {
//     fn chain_id(&self) -> u64 {
//         block::chainid()
//     }
// }

// impl AccountAccess for WasmHost {
//     fn balance(&self, account: Address) -> U256 {
//         account.balance()
//     }
//     fn contract_address(&self) -> Address {
//         contract::address()
//     }
//     fn code(&self, account: Address) -> Vec<u8> {
//         account.code()
//     }
//     fn code_size(&self, account: Address) -> usize {
//         account.code_size()
//     }
//     fn codehash(&self, account: Address) -> FixedBytes<32> {
//         account.code_hash()
//     }
// }

// impl MemoryAccess for WasmHost {
//     fn pay_for_memory_grow(&self, pages: u16) {
//         evm::pay_for_memory_grow(pages);
//     }
// }

// impl MessageAccess for WasmHost {
//     fn msg_sender(&self) -> Address {
//         msg::sender()
//     }
//     fn msg_reentrant(&self) -> bool {
//         msg::reentrant()
//     }
//     fn msg_value(&self) -> U256 {
//         msg::value()
//     }
//     fn tx_origin(&self) -> Address {
//         tx::origin()
//     }
// }

// impl MeteringAccess for WasmHost {
//     fn evm_gas_left(&self) -> u64 {
//         evm::gas_left()
//     }
//     fn evm_ink_left(&self) -> u64 {
//         evm::ink_left()
//     }
//     fn tx_gas_price(&self) -> U256 {
//         tx::gas_price()
//     }
//     fn tx_ink_price(&self) -> u32 {
//         tx::ink_price()
//     }
// }
