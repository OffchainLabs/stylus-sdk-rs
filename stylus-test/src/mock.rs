use alloy_primitives::{address, Address, B256, U256};
use std::{cell::RefCell, collections::HashMap};

pub use stylus_host::*;

/// Arbitrum's CHAID ID.
pub const CHAIN_ID: u64 = 42161;

pub struct TestVM {
    storage: RefCell<HashMap<U256, B256>>,
    msg_sender: Address,
    contract_address: Address,
    chain_id: u64,
    reentrant: bool,
    // Add fields for enhanced testing
    block_number: u64,
    block_timestamp: u64,
    tx_origin: Address,
    balances: RefCell<HashMap<Address, U256>>,
    code_storage: RefCell<HashMap<Address, Vec<u8>>>,
    gas_left: u64,
    ink_left: u64,
}

impl TestVM {
    pub fn new() -> Self {
        Self {
            storage: RefCell::new(HashMap::new()),
            msg_sender: address!("DeaDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF"),
            contract_address: address!("dCE82b5f92C98F27F116F70491a487EFFDb6a2a9"),
            chain_id: CHAIN_ID,
            reentrant: false,
            block_number: 0,
            block_timestamp: 0,
            tx_origin: Address::ZERO,
            balances: RefCell::new(HashMap::new()),
            code_storage: RefCell::new(HashMap::new()),
            gas_left: 1_000_000,
            ink_left: 1_000_000,
        }
    }

    pub fn set_block_number(&mut self, block_number: u64) {
        self.block_number = block_number;
    }

    pub fn set_block_timestamp(&mut self, timestamp: u64) {
        self.block_timestamp = timestamp;
    }

    pub fn set_tx_origin(&mut self, origin: Address) {
        self.tx_origin = origin;
    }

    pub fn set_balance(&mut self, address: Address, balance: U256) {
        self.balances.borrow_mut().insert(address, balance);
    }

    pub fn set_code(&mut self, address: Address, code: Vec<u8>) {
        self.code_storage.borrow_mut().insert(address, code);
    }

    pub fn set_gas_left(&mut self, gas: u64) {
        self.gas_left = gas;
    }

    pub fn set_ink_left(&mut self, ink: u64) {
        self.ink_left = ink;
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

// Update existing trait implementations with new functionality
impl BlockAccess for TestVM {
    fn block_basefee(&self) -> U256 {
        U256::from(1_000_000_000) // Default to 1 gwei.
    }

    fn block_coinbase(&self) -> Address {
        Address::from([0x42; 20])
    }

    fn block_gas_limit(&self) -> u64 {
        30_000_000
    }

    fn block_number(&self) -> u64 {
        self.block_number
    }

    fn block_timestamp(&self) -> u64 {
        self.block_timestamp
    }
}

impl ChainAccess for TestVM {
    fn chain_id(&self) -> u64 {
        self.chain_id
    }
}

impl AccountAccess for TestVM {
    fn balance(&self, account: Address) -> U256 {
        self.balances
            .borrow()
            .get(&account)
            .copied()
            .unwrap_or_default()
    }

    fn code(&self, account: Address) -> Vec<u8> {
        self.code_storage
            .borrow()
            .get(&account)
            .cloned()
            .unwrap_or_default()
    }

    fn code_hash(&self, account: Address) -> B256 {
        if let Some(code) = self.code_storage.borrow().get(&account) {
            alloy_primitives::keccak256(code).into()
        } else {
            B256::ZERO
        }
    }

    fn code_size(&self, account: Address) -> usize {
        self.code_storage
            .borrow()
            .get(&account)
            .map_or(0, |code| code.len())
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
        self.reentrant
    }

    fn msg_sender(&self) -> Address {
        self.msg_sender
    }

    fn msg_value(&self) -> U256 {
        U256::ZERO // Can be enhanced to support value transfers
    }

    fn tx_origin(&self) -> Address {
        self.tx_origin
    }
}

impl MeteringAccess for TestVM {
    fn evm_gas_left(&self) -> u64 {
        self.gas_left
    }

    fn evm_ink_left(&self) -> u64 {
        self.ink_left
    }

    fn tx_gas_price(&self) -> U256 {
        U256::from(1_000_000_000) // Default to 1 gwei
    }

    fn tx_ink_price(&self) -> u32 {
        1_000
    }
}

// Add test helpers
impl TestVM {
    pub fn get_storage(&self, key: U256) -> B256 {
        self.storage.borrow().get(&key).copied().unwrap_or_default()
    }

    pub fn set_storage(&self, key: U256, value: B256) {
        self.storage.borrow_mut().insert(key, value);
    }

    pub fn clear_storage(&self) {
        self.storage.borrow_mut().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_vm_operations() {
        let mut vm = TestVM::new();

        vm.set_block_number(12345);
        assert_eq!(vm.block_number(), 12345);

        let address = Address::from([1u8; 20]);
        let balance = U256::from(1000);
        vm.set_balance(address, balance);
        assert_eq!(vm.balance(address), balance);

        let key = U256::from(1);
        let value = B256::new([1u8; 32]);
        vm.set_storage(key, value);
        assert_eq!(vm.get_storage(key), value);
    }
}
