use alloy_primitives::{Address, B256, U256};
pub use stylus_host::*;

pub struct TestVM {}

impl TestVM {
    pub fn new() -> Self {
        Self {}
    }
}

impl Host for TestVM {}

impl CryptographyAccess for TestVM {
    fn native_keccak256(&self, input: &[u8]) -> B256 {
        B256::ZERO
    }
}
impl CalldataAccess for TestVM {
    fn read_args(&self, len: usize) -> Vec<u8> {
        Vec::new()
    }
    fn read_return_data(&self, offset: usize, size: Option<usize>) -> Vec<u8> {
        Vec::new()
    }
    fn return_data_size(&self) -> usize {
        return 0;
    }
    fn write_result(&self, data: &[u8]) {}
}

unsafe impl DeploymentAccess for TestVM {
    unsafe fn create1(
        &self,
        code: Address,
        endowment: U256,
        contract: &mut Address,
        revert_data_len: &mut usize,
    ) {
    }
    unsafe fn create2(
        &self,
        code: Address,
        endowment: U256,
        salt: B256,
        contract: &mut Address,
        revert_data_len: &mut usize,
    ) {
    }
}
impl StorageAccess for TestVM {
    fn emit_log(&self, input: &[u8], num_topics: usize) {}
    fn flush_cache(&self, clear: bool) {}
    unsafe fn storage_cache_bytes32(&self, key: U256, value: B256) {}
    fn storage_load_bytes32(&self, key: U256) -> B256 {
        B256::ZERO
    }
}
unsafe impl CallAccess for TestVM {
    unsafe fn call_contract(
        &self,
        to: Address,
        data: &[u8],
        value: U256,
        gas: u64,
        outs_len: &mut usize,
    ) -> u8 {
        0
    }
    unsafe fn delegate_call_contract(
        &self,
        to: Address,
        data: &[u8],
        gas: u64,
        outs_len: &mut usize,
    ) -> u8 {
        0
    }
    unsafe fn static_call_contract(
        &self,
        to: Address,
        data: &[u8],
        gas: u64,
        outs_len: &mut usize,
    ) -> u8 {
        0
    }
}
impl BlockAccess for TestVM {
    fn block_basefee(&self) -> U256 {
        U256::ZERO
    }
    fn block_coinbase(&self) -> Address {
        Address::ZERO
    }
    fn block_gas_limit(&self) -> u64 {
        0
    }
    fn block_number(&self) -> u64 {
        0
    }
    fn block_timestamp(&self) -> u64 {
        0
    }
}
impl ChainAccess for TestVM {
    fn chain_id(&self) -> u64 {
        0
    }
}
impl AccountAccess for TestVM {
    fn balance(&self, account: Address) -> U256 {
        U256::ZERO
    }
    fn code(&self, account: Address) -> Vec<u8> {
        Vec::new()
    }
    fn code_hash(&self, account: Address) -> B256 {
        B256::ZERO
    }
    fn code_size(&self, account: Address) -> usize {
        0
    }
    fn contract_address(&self) -> Address {
        Address::ZERO
    }
}
impl MemoryAccess for TestVM {
    fn pay_for_memory_grow(&self, pages: u16) {}
}
impl MessageAccess for TestVM {
    fn msg_reentrant(&self) -> bool {
        false
    }
    fn msg_sender(&self) -> Address {
        Address::ZERO
    }
    fn msg_value(&self) -> U256 {
        U256::ZERO
    }
    fn tx_origin(&self) -> Address {
        Address::ZERO
    }
}
impl MeteringAccess for TestVM {
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
