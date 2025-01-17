use alloy_primitives::{Address, B256, U256};
use std::{cell::RefCell, collections::HashMap};

pub use stylus_host::*;

/// Dummy msg sender set for tests.
pub const MSG_SENDER: &[u8; 42] = b"0xDeaDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF";

/// Dummy contract address set for tests.
pub const CONTRACT_ADDRESS: &[u8; 42] = b"0xdCE82b5f92C98F27F116F70491a487EFFDb6a2a9";

/// Arbitrum's CHAID ID.
pub const CHAIN_ID: u64 = 42161;

pub struct TestVM {
    storage: RefCell<HashMap<U256, B256>>,
    msg_sender: Address,
    contract_address: Address,
    chain_id: u64,
    reentrant: bool,
}

impl TestVM {
    pub fn new() -> Self {
        Self {
            storage: RefCell::new(HashMap::new()),
            msg_sender: Address::from_slice(MSG_SENDER),
            contract_address: Address::from_slice(CONTRACT_ADDRESS),
            chain_id: CHAIN_ID,
            reentrant: false,
        }
    }
    pub fn set_sender(&mut self, sender: Address) {
        self.msg_sender = sender;
    }
    pub fn set_reentrant(&mut self, reentrant: bool) {
        self.reentrant = reentrant;
    }
}

impl Host for TestVM {}

impl CryptographyAccess for TestVM {
    fn native_keccak256(&self, input: &[u8]) -> B256 {
        alloy_primitives::keccak256(input).into()
    }
}
impl CalldataAccess for TestVM {
    fn read_args(&self, _len: usize) -> Vec<u8> {
        Vec::new()
    }
    fn read_return_data(&self, _offset: usize, _size: Option<usize>) -> Vec<u8> {
        Vec::new()
    }
    fn return_data_size(&self) -> usize {
        return 0;
    }
    fn write_result(&self, _data: &[u8]) {}
}

unsafe impl DeploymentAccess for TestVM {
    unsafe fn create1(
        &self,
        _code: Address,
        _endowment: U256,
        _contract: &mut Address,
        _revert_data_len: &mut usize,
    ) {
    }
    unsafe fn create2(
        &self,
        _code: Address,
        _endowment: U256,
        _salt: B256,
        _contract: &mut Address,
        _revert_data_len: &mut usize,
    ) {
    }
}
impl StorageAccess for TestVM {
    fn emit_log(&self, _input: &[u8], _num_topics: usize) {}
    fn flush_cache(&self, _clear: bool) {}
    unsafe fn storage_cache_bytes32(&self, key: U256, value: B256) {
        self.storage.borrow_mut().insert(key, value);
    }
    fn storage_load_bytes32(&self, key: U256) -> B256 {
        self.storage
            .borrow()
            .get(&key)
            .copied()
            .unwrap_or(B256::ZERO)
    }
}
unsafe impl CallAccess for TestVM {
    unsafe fn call_contract(
        &self,
        _to: Address,
        _data: &[u8],
        _value: U256,
        _gas: u64,
        _outs_len: &mut usize,
    ) -> u8 {
        0
    }
    unsafe fn delegate_call_contract(
        &self,
        _to: Address,
        _data: &[u8],
        _gas: u64,
        _outs_len: &mut usize,
    ) -> u8 {
        0
    }
    unsafe fn static_call_contract(
        &self,
        _to: Address,
        _data: &[u8],
        _gas: u64,
        _outs_len: &mut usize,
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
        self.chain_id
    }
}
impl AccountAccess for TestVM {
    fn balance(&self, _account: Address) -> U256 {
        U256::ZERO
    }
    fn code(&self, _account: Address) -> Vec<u8> {
        Vec::new()
    }
    fn code_hash(&self, _account: Address) -> B256 {
        B256::ZERO
    }
    fn code_size(&self, _account: Address) -> usize {
        0
    }
    fn contract_address(&self) -> Address {
        self.contract_address
    }
}
impl MemoryAccess for TestVM {
    fn pay_for_memory_grow(&self, _pages: u16) {}
}
impl MessageAccess for TestVM {
    fn msg_reentrant(&self) -> bool {
        false
    }
    fn msg_sender(&self) -> Address {
        self.msg_sender
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
