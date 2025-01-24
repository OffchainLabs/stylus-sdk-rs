use alloy_primitives::{address, Address, B256, U256};
use calls::{errors::Error, CallAccess, MutatingCallContext, StaticCallContext, ValueTransfer};
use deploy::DeploymentAccess;
use std::{cell::RefCell, collections::HashMap};

pub use stylus_core::*;

/// Arbitrum's CHAID ID.
pub const CHAIN_ID: u64 = 42161;

pub struct TestVM {
    storage: RefCell<HashMap<U256, B256>>,
    msg_sender: Address,
    contract_address: Address,
    chain_id: u64,
    reentrant: bool,
    // Add fields for enhanced testing
    block_number: RefCell<u64>,
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
            block_number: RefCell::new(0),
            block_timestamp: 0,
            tx_origin: Address::ZERO,
            balances: RefCell::new(HashMap::new()),
            code_storage: RefCell::new(HashMap::new()),
            gas_left: 1_000_000,
            ink_left: 1_000_000,
        }
    }
}

pub trait TestHost: Host {
    fn set_block_number(&self, block_number: u64);
    fn set_block_timestamp(&self, timestamp: u64);
    fn set_tx_origin(&self, origin: Address);
    fn set_balance(&self, address: Address, balance: U256);
    fn set_code(&self, address: Address, code: Vec<u8>);
    fn set_gas_left(&self, gas: u64);
    fn set_ink_left(&self, ink: u64);
}

impl TestHost for TestVM {
    fn set_block_number(&self, block_number: u64) {
        *self.block_number.borrow_mut() = block_number;
    }
    fn set_block_timestamp(&self, timestamp: u64) {}
    fn set_tx_origin(&self, origin: Address) {}
    fn set_balance(&self, address: Address, balance: U256) {
        self.balances.borrow_mut().insert(address, balance);
    }
    fn set_code(&self, address: Address, code: Vec<u8>) {
        self.code_storage.borrow_mut().insert(address, code);
    }
    fn set_gas_left(&self, gas: u64) {}
    fn set_ink_left(&self, ink: u64) {}
}

impl Default for TestVM {
    fn default() -> Self {
        Self::new()
    }
}

impl Host for TestVM {}

impl CryptographyAccess for TestVM {
    fn native_keccak256(&self, input: &[u8]) -> B256 {
        alloy_primitives::keccak256(input)
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
        0
    }
    fn write_result(&self, _data: &[u8]) {}
}

unsafe impl UnsafeDeploymentAccess for TestVM {
    unsafe fn create1(
        &self,
        _code: *const u8,
        _code_len: usize,
        _endowment: *const u8,
        _contract: *mut u8,
        _revert_data_len: *mut usize,
    ) {
    }
    unsafe fn create2(
        &self,
        _code: *const u8,
        _code_len: usize,
        _endowment: *const u8,
        _salt: *const u8,
        _contract: *mut u8,
        _revert_data_len: *mut usize,
    ) {
    }
}

impl StorageAccess for TestVM {
    unsafe fn storage_cache_bytes32(&self, key: U256, value: B256) {
        self.storage.borrow_mut().insert(key, value);
    }

    fn emit_log(&self, _input: &[u8], _num_topics: usize) {}
    fn flush_cache(&self, _clear: bool) {}
    fn storage_load_bytes32(&self, key: U256) -> B256 {
        self.storage
            .borrow()
            .get(&key)
            .copied()
            .unwrap_or(B256::ZERO)
    }
}

unsafe impl UnsafeCallAccess for TestVM {
    unsafe fn call_contract(
        &self,
        _to: *const u8,
        _data: *const u8,
        _data_len: usize,
        _value: *const u8,
        _gas: u64,
        _outs_len: &mut usize,
    ) -> u8 {
        0
    }
    unsafe fn delegate_call_contract(
        &self,
        _to: *const u8,
        _data: *const u8,
        _data_len: usize,
        _gas: u64,
        _outs_len: &mut usize,
    ) -> u8 {
        0
    }
    unsafe fn static_call_contract(
        &self,
        _to: *const u8,
        _data: *const u8,
        _data_len: usize,
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
        *self.block_number.borrow()
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
            alloy_primitives::keccak256(code)
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

impl CallAccess for TestVM {
    fn call(
        &self,
        _context: &dyn MutatingCallContext,
        _to: Address,
        _data: &[u8],
    ) -> Result<Vec<u8>, Error> {
        Ok(Vec::new())
    }
    unsafe fn delegate_call(
        &self,
        _context: &dyn MutatingCallContext,
        _to: Address,
        _data: &[u8],
    ) -> Result<Vec<u8>, Error> {
        Ok(Vec::new())
    }
    fn static_call(
        &self,
        _context: &dyn StaticCallContext,
        _to: Address,
        _data: &[u8],
    ) -> Result<Vec<u8>, Error> {
        Ok(Vec::new())
    }
}

impl ValueTransfer for TestVM {
    #[cfg(feature = "reentrant")]
    fn transfer_eth(
        &self,
        _storage: &mut dyn stylus_core::storage::TopLevelStorage,
        _to: Address,
        _amount: U256,
    ) -> Result<(), Vec<u8>> {
        Ok(())
    }
    #[cfg(not(feature = "reentrant"))]
    fn transfer_eth(&self, _to: Address, _amount: U256) -> Result<(), Vec<u8>> {
        Ok(())
    }
}

impl DeploymentAccess for TestVM {
    #[cfg(feature = "reentrant")]
    unsafe fn deploy(
        &self,
        _code: &[u8],
        _endowment: U256,
        _salt: Option<B256>,
        _cache_policy: stylus_core::deploy::CachePolicy,
    ) -> Result<Address, Vec<u8>> {
        Ok(Address::ZERO)
    }
    #[cfg(not(feature = "reentrant"))]
    unsafe fn deploy(
        &self,
        _code: &[u8],
        _endowment: U256,
        _salt: Option<B256>,
    ) -> Result<Address, Vec<u8>> {
        Ok(Address::ZERO)
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
