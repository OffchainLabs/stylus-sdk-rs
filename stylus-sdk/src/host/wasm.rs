use alloy_primitives::Address;
use once_cell::sync::Lazy;

use crate::{block, contract, evm, host::*, msg, tx, types::AddressVM};

pub static GLOBAL_WASM_HOST: Lazy<WasmHost> = Lazy::new(|| WasmHost {});

pub struct WasmHost;

impl Host for WasmHost {}

impl CryptographyAccess for WasmHost {
    fn native_keccak256(&self, input: &[u8]) -> FixedBytes<32> {
        FixedBytes::<32>::default()
    }
}

impl CalldataAccess for WasmHost {
    fn read_args(&self) -> Vec<u8> {
        Vec::new()
    }
    fn read_return_data(&self) -> Vec<u8> {
        Vec::new()
    }
    fn return_data_size(&self) -> usize {
        0
    }
    fn write_result(&self) {}
}

impl DeploymentAccess for WasmHost {
    fn create1(&self) {}
    fn create2(&self) {}
}

impl StorageAccess for WasmHost {
    fn emit_log(&self, input: &[u8]) {}
    fn load(&self, key: FixedBytes<32>) -> FixedBytes<32> {
        FixedBytes::<32>::default()
    }
    fn cache(&self, key: FixedBytes<32>, value: FixedBytes<32>) {}
    fn flush_cache(&self, clear: bool) {}
}

impl CallAccess for WasmHost {
    fn call_contract(&self) {}
    fn static_call_contract(&self) {}
    fn delegate_call_contract(&self) {}
}

impl BlockAccess for WasmHost {
    fn block_basefee(&self) -> U256 {
        block::basefee()
    }
    fn block_coinbase(&self) -> Address {
        block::coinbase()
    }
    fn block_number(&self) -> u64 {
        block::number()
    }
    fn block_timestamp(&self) -> u64 {
        block::timestamp()
    }
    fn block_gas_limit(&self) -> u64 {
        block::gas_limit()
    }
}

impl ChainAccess for WasmHost {
    fn chain_id(&self) -> u64 {
        block::chainid()
    }
}

impl AccountAccess for WasmHost {
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
    fn codehash(&self, account: Address) -> FixedBytes<32> {
        account.code_hash()
    }
}

impl MemoryAccess for WasmHost {
    fn pay_for_memory_grow(&self, pages: u16) {}
}

impl MessageAccess for WasmHost {
    fn msg_sender(&self) -> Address {
        msg::sender()
    }
    fn msg_reentrant(&self) -> bool {
        msg::reentrant()
    }
    fn msg_value(&self) -> U256 {
        msg::value()
    }
    fn tx_origin(&self) -> Address {
        tx::origin()
    }
}

impl MeteringAccess for WasmHost {
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
