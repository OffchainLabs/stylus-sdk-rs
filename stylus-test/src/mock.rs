use alloy_primitives::{address, Address, B256, U256};
use calls::{errors::Error, CallAccess, MutatingCallContext, StaticCallContext, ValueTransfer};
use deploy::DeploymentAccess;
use ethers::middleware::Middleware;
use ethers::{
    providers::{Http, Provider},
    types::{NameOrAddress, H160, H256},
};
use rclite::Rc;
use std::{cell::RefCell, collections::HashMap, sync::Arc};
use tokio::runtime::Runtime;

pub use stylus_core::*;

/// Arbitrum's CHAIN ID.
pub const CHAIN_ID: u64 = 42161;

#[derive(Clone)]
pub(crate) struct MockVMState {
    pub storage: HashMap<U256, B256>,
    pub msg_sender: Address,
    pub contract_address: Address,
    pub chain_id: u64,
    pub reentrant: bool,
    // Add fields for enhanced testing
    pub block_number: u64,
    pub block_timestamp: u64,
    pub tx_origin: Address,
    pub balances: HashMap<Address, U256>,
    pub code_storage: HashMap<Address, Vec<u8>>,
    pub gas_left: u64,
    pub ink_left: u64,
    pub msg_value: U256,
    pub block_gas_limit: u64,
    pub coinbase: Address,
    pub block_basefee: U256,
    pub tx_gas_price: U256,
    pub tx_ink_price: u32,
    // Add ways of mocking expected calls, deployments, and logs.
    pub call_returns: HashMap<(Address, Vec<u8>), Result<Vec<u8>, Vec<u8>>>,
    pub delegate_call_returns: HashMap<(Address, Vec<u8>), Result<Vec<u8>, Vec<u8>>>,
    pub static_call_returns: HashMap<(Address, Vec<u8>), Result<Vec<u8>, Vec<u8>>>,
    pub deploy_returns: HashMap<(Vec<u8>, Option<B256>), Result<Address, Vec<u8>>>,
    pub emitted_logs: Vec<(Vec<B256>, Vec<u8>)>,
    pub provider: Option<Arc<Provider<Http>>>,
}

impl MockVMState {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
            msg_sender: address!("DeaDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF"),
            contract_address: address!("dCE82b5f92C98F27F116F70491a487EFFDb6a2a9"),
            chain_id: CHAIN_ID,
            reentrant: false,
            block_number: 0,
            block_timestamp: 0,
            tx_origin: Address::ZERO,
            balances: HashMap::new(),
            code_storage: HashMap::new(),
            gas_left: 1_000_000,
            ink_left: 1_000_000,
            msg_value: U256::ZERO,
            block_basefee: U256::from(1_000_000),
            block_gas_limit: 30_000_000,
            coinbase: address!("DeaDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF"),
            tx_gas_price: U256::from(1),
            tx_ink_price: 1,
            call_returns: HashMap::new(),
            delegate_call_returns: HashMap::new(),
            static_call_returns: HashMap::new(),
            deploy_returns: HashMap::new(),
            emitted_logs: Vec::new(),
            provider: None,
        }
    }
}

#[derive(Clone)]
pub struct TestVM {
    state: Rc<RefCell<MockVMState>>,
}

impl Default for TestVM {
    fn default() -> Self {
        Self::new()
    }
}

impl From<MockVMState> for TestVM {
    fn from(state: MockVMState) -> Self {
        Self {
            state: Rc::new(RefCell::new(state)),
        }
    }
}

impl TestVM {
    pub fn new() -> Self {
        Self {
            state: Rc::new(RefCell::new(MockVMState::new())),
        }
    }
    pub fn set_block_number(&self, block_number: u64) {
        self.state.borrow_mut().block_number = block_number;
    }
    pub fn set_block_timestamp(&self, timestamp: u64) {
        self.state.borrow_mut().block_timestamp = timestamp;
    }
    pub fn set_tx_origin(&self, origin: Address) {
        self.state.borrow_mut().tx_origin = origin;
    }
    pub fn set_balance(&self, address: Address, balance: U256) {
        self.state.borrow_mut().balances.insert(address, balance);
    }
    pub fn set_contract_address(&self, address: Address) {
        self.state.borrow_mut().contract_address = address;
    }
    pub fn set_code(&self, address: Address, code: Vec<u8>) {
        self.state.borrow_mut().code_storage.insert(address, code);
    }
    pub fn set_gas_left(&self, gas: u64) {
        self.state.borrow_mut().gas_left = gas;
    }
    pub fn set_ink_left(&self, ink: u64) {
        self.state.borrow_mut().ink_left = ink;
    }
    pub fn set_sender(&self, sender: Address) {
        self.state.borrow_mut().msg_sender = sender;
    }
    pub fn set_value(&self, value: U256) {
        self.state.borrow_mut().msg_value = value;
    }
    pub fn get_storage(&self, key: U256) -> B256 {
        self.state
            .borrow()
            .storage
            .get(&key)
            .copied()
            .unwrap_or_default()
    }
    pub fn set_storage(&self, key: U256, value: B256) {
        self.state.borrow_mut().storage.insert(key, value);
    }
    pub fn clear_storage(&self) {
        self.state.borrow_mut().storage.clear();
    }
    pub fn mock_call(&self, to: Address, data: Vec<u8>, return_data: Result<Vec<u8>, Vec<u8>>) {
        self.state
            .borrow_mut()
            .call_returns
            .insert((to, data), return_data);
    }
    pub fn mock_delegate_call(
        &self,
        to: Address,
        data: Vec<u8>,
        return_data: Result<Vec<u8>, Vec<u8>>,
    ) {
        self.state
            .borrow_mut()
            .delegate_call_returns
            .insert((to, data), return_data);
    }
    pub fn mock_static_call(
        &self,
        to: Address,
        data: Vec<u8>,
        return_data: Result<Vec<u8>, Vec<u8>>,
    ) {
        self.state
            .borrow_mut()
            .static_call_returns
            .insert((to, data), return_data);
    }
    pub fn mock_deploy(&self, code: Vec<u8>, salt: Option<B256>, result: Result<Address, Vec<u8>>) {
        self.state
            .borrow_mut()
            .deploy_returns
            .insert((code, salt), result);
    }
    pub fn get_emitted_logs(&self) -> Vec<(Vec<B256>, Vec<u8>)> {
        self.state.borrow().emitted_logs.clone()
    }
    pub fn clear_mocks(&self) {
        let mut state = self.state.borrow_mut();
        state.call_returns.clear();
        state.delegate_call_returns.clear();
        state.static_call_returns.clear();
        state.deploy_returns.clear();
        state.emitted_logs.clear();
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
        self.state.borrow_mut().storage.insert(key, value);
    }

    fn flush_cache(&self, _clear: bool) {}
    fn storage_load_bytes32(&self, key: U256) -> B256 {
        if let Some(provider) = self.state.borrow().provider.clone() {
            let rt = Runtime::new().expect("Failed to create runtime");

            let slot_bytes: &[u8; 32] = &key.to_be_bytes();
            let slot = H256::from_slice(&slot_bytes[..]);

            let addr = NameOrAddress::Address(H160::from_slice(
                &self.state.borrow().contract_address.as_slice(),
            ));
            let storage = rt
                .block_on(async { provider.get_storage_at(addr, slot, None).await })
                .unwrap_or_default();
            return B256::from_slice(storage.as_bytes());
        }
        self.state
            .borrow()
            .storage
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
        self.state.borrow().block_basefee
    }

    fn block_coinbase(&self) -> Address {
        self.state.borrow().coinbase
    }

    fn block_gas_limit(&self) -> u64 {
        self.state.borrow().block_gas_limit
    }

    fn block_number(&self) -> u64 {
        self.state.borrow().block_number
    }

    fn block_timestamp(&self) -> u64 {
        self.state.borrow().block_timestamp
    }
}

impl ChainAccess for TestVM {
    fn chain_id(&self) -> u64 {
        self.state.borrow().chain_id
    }
}

impl AccountAccess for TestVM {
    fn balance(&self, account: Address) -> U256 {
        self.state
            .borrow()
            .balances
            .get(&account)
            .copied()
            .unwrap_or_default()
    }

    fn code(&self, account: Address) -> Vec<u8> {
        self.state
            .borrow()
            .code_storage
            .get(&account)
            .cloned()
            .unwrap_or_default()
    }

    fn code_hash(&self, account: Address) -> B256 {
        if let Some(code) = self.state.borrow().code_storage.get(&account) {
            alloy_primitives::keccak256(code)
        } else {
            B256::ZERO
        }
    }

    fn code_size(&self, account: Address) -> usize {
        self.state
            .borrow()
            .code_storage
            .get(&account)
            .map_or(0, |code| code.len())
    }

    fn contract_address(&self) -> Address {
        self.state.borrow().contract_address
    }
}

impl MemoryAccess for TestVM {
    fn pay_for_memory_grow(&self, _pages: u16) {}
}

impl MessageAccess for TestVM {
    fn msg_reentrant(&self) -> bool {
        self.state.borrow().reentrant
    }

    fn msg_sender(&self) -> Address {
        self.state.borrow().msg_sender
    }

    fn msg_value(&self) -> U256 {
        self.state.borrow().msg_value
    }

    fn tx_origin(&self) -> Address {
        self.state.borrow().tx_origin
    }
}

impl MeteringAccess for TestVM {
    fn evm_gas_left(&self) -> u64 {
        self.state.borrow().gas_left
    }

    fn evm_ink_left(&self) -> u64 {
        self.state.borrow().ink_left
    }

    fn tx_gas_price(&self) -> U256 {
        self.state.borrow().tx_gas_price
    }

    fn tx_ink_price(&self) -> u32 {
        self.state.borrow().tx_ink_price
    }
}

impl CallAccess for TestVM {
    fn call(
        &self,
        _context: &dyn MutatingCallContext,
        to: Address,
        data: &[u8],
    ) -> Result<Vec<u8>, Error> {
        self.state
            .borrow()
            .call_returns
            .get(&(to, data.to_vec()))
            .cloned()
            .map(|opt| match opt {
                Ok(data) => Ok(data),
                Err(data) => Err(Error::Revert(data)),
            })
            .unwrap_or(Ok(Vec::new()))
    }

    unsafe fn delegate_call(
        &self,
        _context: &dyn MutatingCallContext,
        to: Address,
        data: &[u8],
    ) -> Result<Vec<u8>, Error> {
        self.state
            .borrow()
            .delegate_call_returns
            .get(&(to, data.to_vec()))
            .cloned()
            .map(|opt| match opt {
                Ok(data) => Ok(data),
                Err(data) => Err(Error::Revert(data)),
            })
            .unwrap_or(Ok(Vec::new()))
    }

    fn static_call(
        &self,
        _context: &dyn StaticCallContext,
        to: Address,
        data: &[u8],
    ) -> Result<Vec<u8>, Error> {
        self.state
            .borrow()
            .static_call_returns
            .get(&(to, data.to_vec()))
            .cloned()
            .map(|opt| match opt {
                Ok(data) => Ok(data),
                Err(data) => Err(Error::Revert(data)),
            })
            .unwrap_or(Ok(Vec::new()))
    }
}

impl ValueTransfer for TestVM {
    #[cfg(feature = "reentrant")]
    fn transfer_eth(
        &self,
        _storage: &mut dyn stylus_core::storage::TopLevelStorage,
        to: Address,
        amount: U256,
    ) -> Result<(), Vec<u8>> {
        let mut state = self.state.borrow_mut();
        let from = state.contract_address;

        let from_balance = state.balances.get(&from).copied().unwrap_or_default();
        let to_balance = state.balances.get(&to).copied().unwrap_or_default();

        if from_balance < amount {
            return Err(b"insufficient funds for transfer".to_vec());
        }

        let new_to_balance = to_balance
            .checked_add(amount)
            .ok_or_else(|| b"balance overflow".to_vec())?;

        state.balances.insert(from, from_balance - amount);
        state.balances.insert(to, new_to_balance);

        Ok(())
    }

    #[cfg(not(feature = "reentrant"))]
    fn transfer_eth(&self, to: Address, amount: U256) -> Result<(), Vec<u8>> {
        let mut state = self.state.borrow_mut();
        let from = state.contract_address;

        let from_balance = state.balances.get(&from).copied().unwrap_or_default();
        let to_balance = state.balances.get(&to).copied().unwrap_or_default();

        if from_balance < amount {
            return Err(b"insufficient funds for transfer".to_vec());
        }

        let new_to_balance = to_balance
            .checked_add(amount)
            .ok_or_else(|| b"balance overflow".to_vec())?;

        state.balances.insert(from, from_balance - amount);
        state.balances.insert(to, new_to_balance);

        Ok(())
    }
}

impl DeploymentAccess for TestVM {
    #[cfg(feature = "reentrant")]
    unsafe fn deploy(
        &self,
        code: &[u8],
        _endowment: U256,
        salt: Option<B256>,
        _cache_policy: stylus_core::deploy::CachePolicy,
    ) -> Result<Address, Vec<u8>> {
        self.state
            .borrow()
            .deploy_returns
            .get(&(code.to_vec(), salt))
            .cloned()
            .unwrap_or(Ok(Address::ZERO))
    }

    #[cfg(not(feature = "reentrant"))]
    unsafe fn deploy(
        &self,
        code: &[u8],
        _endowment: U256,
        salt: Option<B256>,
    ) -> Result<Address, Vec<u8>> {
        self.state
            .borrow()
            .deploy_returns
            .get(&(code.to_vec(), salt))
            .cloned()
            .unwrap_or(Ok(Address::ZERO))
    }
}

impl LogAccess for TestVM {
    fn emit_log(&self, input: &[u8], num_topics: usize) {
        let (topics_data, data) = input.split_at(num_topics * 32);
        let mut topics = Vec::with_capacity(num_topics);

        for chunk in topics_data.chunks(32) {
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(chunk);
            topics.push(B256::from(bytes));
        }

        self.state
            .borrow_mut()
            .emitted_logs
            .push((topics, data.to_vec()));
    }

    fn raw_log(&self, topics: &[B256], data: &[u8]) -> Result<(), &'static str> {
        self.state
            .borrow_mut()
            .emitted_logs
            .push((topics.to_vec(), data.to_vec()));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_vm_operations() {
        let vm = TestVM::new();

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

    #[test]
    fn test_mock_calls() {
        let vm = TestVM::new();
        let target = Address::from([2u8; 20]);
        let data = vec![1, 2, 3, 4];
        let expected_return = vec![5, 6, 7, 8];

        // Mock a regular call.
        vm.mock_call(target, data.clone(), Ok(expected_return.clone()));

        let ctx = stylus_core::calls::context::Call::new();
        let result = vm.call(&ctx, target, &data).unwrap();
        assert_eq!(result, expected_return);

        // Mock an error case.
        let error_data = vec![9, 9, 9];
        vm.mock_call(target, data.clone(), Err(error_data.clone()));

        match vm.call(&ctx, target, &data) {
            Err(Error::Revert(returned_data)) => assert_eq!(returned_data, error_data),
            _ => panic!("Expected revert error"),
        }
    }

    #[test]
    fn test_mock_deploys() {
        let vm = TestVM::new();
        let code = vec![1, 2, 3, 4];
        let expected_address = Address::from([3u8; 20]);

        // Mock a successful deployment.
        vm.mock_deploy(code.clone(), None, Ok(expected_address));

        unsafe {
            let result = vm.deploy(&code, U256::ZERO, None).unwrap();
            assert_eq!(result, expected_address);
        }

        // Mock a failed deployment.
        let error_data = vec![9, 9, 9];
        vm.mock_deploy(code.clone(), None, Err(error_data.clone()));

        unsafe {
            match vm.deploy(&code, U256::ZERO, None) {
                Err(returned_data) => assert_eq!(returned_data, error_data),
                _ => panic!("Expected deployment error"),
            }
        }
    }

    #[test]
    fn test_logs() {
        let vm = TestVM::new();
        let topic1 = B256::from([1u8; 32]);
        let topic2 = B256::from([2u8; 32]);
        let data = vec![3, 4, 5];

        vm.raw_log(&[topic1, topic2], &data).unwrap();

        let logs = vm.get_emitted_logs();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].0, vec![topic1, topic2]);
        assert_eq!(logs[0].1, data);
    }

    #[test]
    fn test_transfer_eth_success() {
        let vm = TestVM::new();
        let from = vm.state.borrow().contract_address;
        let to = Address::from([1u8; 20]);
        let initial_balance = U256::from(1000);
        let transfer_amount = U256::from(300);

        vm.set_balance(from, initial_balance);

        let result = vm.transfer_eth(to, transfer_amount);
        assert!(result.is_ok());

        assert_eq!(vm.balance(from), initial_balance - transfer_amount);
        assert_eq!(vm.balance(to), transfer_amount);
    }

    #[test]
    fn test_transfer_eth_insufficient_funds() {
        let vm = TestVM::new();
        let from = vm.state.borrow().contract_address;
        let to = Address::from([1u8; 20]);
        let initial_balance = U256::from(100);
        let transfer_amount = U256::from(200);

        vm.set_balance(from, initial_balance);

        let result = vm.transfer_eth(to, transfer_amount);
        assert!(result.is_err());

        // Check that balances remain unchanged
        assert_eq!(vm.balance(from), initial_balance);
        assert_eq!(vm.balance(to), U256::ZERO);
    }

    #[test]
    fn test_transfer_eth_overflow() {
        let vm = TestVM::new();
        let from = vm.state.borrow().contract_address;
        let to = Address::from([1u8; 20]);

        vm.set_balance(from, U256::MAX);
        vm.set_balance(to, U256::MAX);

        let result = vm.transfer_eth(to, U256::from(1));
        assert!(result.is_err());

        assert_eq!(vm.balance(from), U256::MAX);
        assert_eq!(vm.balance(to), U256::MAX);
    }
}
