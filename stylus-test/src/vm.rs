// Copyright 2025-2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Defines a test VM environment for unit testing Stylus contracts.
//! Allows for mocking of all host methods defined on the [`stylus_core::host::Host`] trait, such
//! as access to storage, messages, block values, and more.
//!
//! ```ignore
//! use stylus_sdk::{alloy_primitives::U256, prelude::*};
//!
//! #[entrypoint]
//! #[storage]
//! pub struct MyContract;
//!
//! #[public]
//! impl MyContract {
//!     pub fn check_msg_value(&self) -> U256 {
//!        self.vm().msg_value()
//!     }
//! }
//!
//! #[cfg(test)]
//! mod test {
//!     use super::*;
//!     use stylus_test::*;
//!
//!     #[test]
//!     fn test_my_contract() {
//!         let vm = TestVM::default();
//!         let contract = MyContract::from(&vm);
//!
//!         let want = U256::from(100);
//!         vm.set_value(want);
//!         let got = contract.check_msg_value();
//!
//!         assert_eq!(want, got);
//!     }
//! }
//! ```

use alloy_primitives::{Address, B256, U256};
use alloy_provider::Provider;
pub use calls::{errors::Error, MutatingCallContext, StaticCallContext};
use std::cell::RefCell;
use std::rc::Rc;
use std::slice;
use tokio::runtime::Runtime;

pub use stylus_core::*;

use crate::state::VMState;

/// A mock implementation of the [`stylus_core::host::Host`] trait for unit testing Stylus contracts.
///
/// # Examples
/// ```
/// use stylus_test::TestVM;
/// use alloy_primitives::{Address, U256};
///
/// let vm = TestVM::new();
///
/// // Configure transaction state.
/// vm.set_block_number(100);
/// vm.set_sender(Address::from([1u8; 20]));
/// vm.set_value(U256::from(1000));
///
/// // Mock contract calls.
/// let contract = Address::from([2u8; 20]);
/// let data = vec![0x01, 0x02, 0x03];
/// vm.mock_call(contract, data.clone(), U256::from(1000), Ok(vec![0x04]));
///
/// // Get emitted logs after execution
/// let logs = vm.get_emitted_logs();
/// ```
#[derive(Clone)]
pub struct TestVM {
    state: Rc<RefCell<VMState>>,
}

impl Default for TestVM {
    fn default() -> Self {
        Self::new()
    }
}

impl From<VMState> for TestVM {
    fn from(state: VMState) -> Self {
        Self {
            state: Rc::new(RefCell::new(state)),
        }
    }
}

impl TestVM {
    /// Creates a new TestVM instance.
    ///
    /// # Examples
    /// ```
    /// use stylus_test::TestVM;
    /// let vm = TestVM::new();
    /// ```
    pub fn new() -> Self {
        Self {
            state: Rc::new(RefCell::new(VMState::default())),
        }
    }

    /// Returns a cloned snapshot of the internal test VM state,
    /// which contains storage, balances, and other mocked values
    /// in HashMaps and other simple data structures for inspection.
    pub fn snapshot(&self) -> VMState {
        self.state.borrow().clone()
    }

    /// Sets the current block number.
    ///
    /// # Examples
    /// ```
    /// # use stylus_test::TestVM;
    /// let vm = TestVM::new();
    /// vm.set_block_number(15_000_000);
    /// ```
    pub fn set_block_number(&self, block_number: u64) {
        self.state.borrow_mut().block_number = block_number;
    }

    /// Sets the current block timestamp.
    ///
    /// # Examples
    /// ```
    /// # use stylus_test::TestVM;
    /// let vm = TestVM::new();
    /// vm.set_block_timestamp(1677654321);
    /// ```
    pub fn set_block_timestamp(&self, timestamp: u64) {
        self.state.borrow_mut().block_timestamp = timestamp;
    }

    /// Sets the transaction origin address.
    pub fn set_tx_origin(&self, origin: Address) {
        self.state.borrow_mut().tx_origin = Some(origin);
    }

    /// Sets the balance for an address.
    ///
    /// # Examples
    /// ```
    /// # use stylus_test::TestVM;
    /// # use alloy_primitives::{Address, U256};
    /// let vm = TestVM::new();
    /// let addr = Address::from([1u8; 20]);
    /// vm.set_balance(addr, U256::from(1000));
    /// ```
    pub fn set_balance(&self, address: Address, balance: U256) {
        self.state.borrow_mut().balances.insert(address, balance);
    }

    /// Sets the contract address.
    pub fn set_contract_address(&self, address: Address) {
        self.state.borrow_mut().contract_address = address;
    }

    /// Sets contract bytecode at an address.
    pub fn set_code(&self, address: Address, code: Vec<u8>) {
        self.state.borrow_mut().code_storage.insert(address, code);
    }

    /// Sets remaining gas.
    pub fn set_gas_left(&self, gas: u64) {
        self.state.borrow_mut().gas_left = gas;
    }

    /// Sets remaining ink.
    pub fn set_ink_left(&self, ink: u64) {
        self.state.borrow_mut().ink_left = ink;
    }

    /// Sets the chain id.
    pub fn set_chain_id(&self, id: u64) {
        self.state.borrow_mut().chain_id = id;
    }

    /// Sets the transaction sender.
    pub fn set_sender(&self, sender: Address) {
        self.state.borrow_mut().msg_sender = sender;
    }

    /// Sets the transaction value.
    pub fn set_value(&self, value: U256) {
        self.state.borrow_mut().msg_value = value;
    }

    /// Gets a storage value by key.
    ///
    /// # Examples
    /// ```
    /// # use stylus_test::TestVM;
    /// # use alloy_primitives::{B256, U256};
    /// let vm = TestVM::new();
    /// let key = U256::from(1);
    /// let value = vm.get_storage(key);
    /// assert_eq!(value, B256::ZERO);
    /// ```
    pub fn get_storage(&self, key: U256) -> B256 {
        self.state
            .borrow()
            .storage
            .get(&key)
            .copied()
            .unwrap_or_default()
    }

    /// Sets a storage value.
    pub fn set_storage(&self, key: U256, value: B256) {
        self.state.borrow_mut().storage.insert(key, value);
    }

    /// Clears all storage.
    pub fn clear_storage(&self) {
        self.state.borrow_mut().storage.clear();
    }

    /// Mocks a contract call.
    ///
    /// # Examples
    /// ```
    /// # use stylus_test::TestVM;
    /// # use alloy_primitives::{Address, U256};
    /// let vm = TestVM::new();
    /// let contract = Address::from([1u8; 20]);
    /// let data = vec![0x01, 0x02, 0x03];
    ///
    /// // Mock successful call
    /// let value = U256::from(1);
    /// vm.mock_call(contract, data.clone(), value, Ok(vec![0x04]));
    ///
    /// let value = U256::from(0);
    /// // Mock reverted call
    /// vm.mock_call(contract, data, value, Err(vec![0xff]));
    /// ```
    pub fn mock_call(
        &self,
        to: Address,
        data: Vec<u8>,
        value: U256,
        return_data: Result<Vec<u8>, Vec<u8>>,
    ) {
        let mut state = self.state.borrow_mut();
        state.return_data = match return_data.clone() {
            Ok(data) => data,
            Err(data) => data,
        };
        state.call_returns.insert((to, data, value), return_data);
    }

    /// Mocks a delegate call.
    pub fn mock_delegate_call(
        &self,
        to: Address,
        data: Vec<u8>,
        return_data: Result<Vec<u8>, Vec<u8>>,
    ) {
        let mut state = self.state.borrow_mut();
        state.return_data = match return_data.clone() {
            Ok(data) => data,
            Err(data) => data,
        };

        state.delegate_call_returns.insert((to, data), return_data);
    }

    /// Mocks a static call.
    pub fn mock_static_call(
        &self,
        to: Address,
        data: Vec<u8>,
        return_data: Result<Vec<u8>, Vec<u8>>,
    ) {
        let mut state = self.state.borrow_mut();
        state.return_data = match return_data.clone() {
            Ok(data) => data,
            Err(data) => data,
        };
        state.static_call_returns.insert((to, data), return_data);
    }

    /// Mocks contract deployment.
    ///
    /// # Examples
    /// ```
    /// # use stylus_test::TestVM;
    /// # use alloy_primitives::{Address, B256};
    /// let vm = TestVM::new();
    /// let code = vec![0x60, 0x80, 0x60, 0x40];
    /// let salt = Some(B256::with_last_byte(1));
    /// let deployed_address = Address::from([2u8; 20]);
    ///
    /// vm.mock_deploy(code, salt, Ok(deployed_address));
    /// ```
    pub fn mock_deploy(&self, code: Vec<u8>, salt: Option<B256>, result: Result<Address, Vec<u8>>) {
        self.state
            .borrow_mut()
            .deploy_returns
            .insert((code, salt), result);
    }

    /// Gets all emitted logs.
    pub fn get_emitted_logs(&self) -> Vec<(Vec<B256>, Vec<u8>)> {
        self.state.borrow().emitted_logs.clone()
    }

    /// Clears all mocks and logs.
    pub fn clear_mocks(&self) {
        let mut state = self.state.borrow_mut();
        state.call_returns.clear();
        state.delegate_call_returns.clear();
        state.static_call_returns.clear();
        state.deploy_returns.clear();
        state.emitted_logs.clear();
        state.return_data.clear();
    }

    fn perform_mocked_call(
        &self,
        to: Address,
        data: Vec<u8>,
        value: U256,
    ) -> Result<Vec<u8>, Vec<u8>> {
        let state = self.state.borrow();
        if let Some(return_data) = state.call_returns.get(&(to, data.clone(), value)) {
            return return_data.clone();
        }
        Ok(Vec::new())
    }

    /// Performs a mocked call to a contract.
    fn perform_mocked_delegate_call(&self, to: Address, data: Vec<u8>) -> Result<Vec<u8>, Vec<u8>> {
        let state = self.state.borrow();
        if let Some(return_data) = state.delegate_call_returns.get(&(to, data.clone())) {
            return return_data.clone();
        }
        Ok(Vec::new())
    }

    /// Performs a mocked call to a contract.
    fn perform_mocked_static_call(&self, to: Address, data: Vec<u8>) -> Result<Vec<u8>, Vec<u8>> {
        let state = self.state.borrow();
        if let Some(return_data) = state.static_call_returns.get(&(to, data.clone())) {
            return return_data.clone();
        }
        Ok(Vec::new())
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
        unimplemented!("read_args not yet implemented for TestVM")
    }

    fn read_return_data(&self, offset: usize, size: Option<usize>) -> Vec<u8> {
        let state = self.state.borrow();
        let data = &state.return_data;
        let start = offset.min(data.len());
        let end = match size {
            Some(s) => (start + s).min(data.len()),
            None => data.len(),
        };
        data[start..end].to_vec()
    }

    fn return_data_size(&self) -> usize {
        self.state.borrow().return_data.len()
    }

    fn write_result(&self, data: &[u8]) {
        let mut state = self.state.borrow_mut();
        state.return_data.clear();
        state.return_data.extend_from_slice(data);
    }
}

unsafe impl UnsafeDeploymentAccess for TestVM {
    unsafe fn create1(
        &self,
        code: *const u8,
        code_len: usize,
        _endowment: *const u8,
        _contract: *mut u8,
        _revert_data_len: *mut usize,
    ) {
        let code = slice::from_raw_parts(code, code_len);
        let deployment_result = self
            .state
            .borrow()
            .deploy_returns
            .get(&(code.to_vec(), None))
            .cloned();
        if deployment_result.is_none() {
            return;
        }
        if let Some(Ok(addr)) = deployment_result {
            let contract = slice::from_raw_parts_mut(_contract, 20);
            contract.copy_from_slice(addr.as_ref());
        }
    }
    unsafe fn create2(
        &self,
        code: *const u8,
        code_len: usize,
        _endowment: *const u8,
        salt: *const u8,
        _contract: *mut u8,
        _revert_data_len: *mut usize,
    ) {
        let salt = slice::from_raw_parts(salt, 32);
        let salt = B256::from_slice(salt);
        let code = slice::from_raw_parts(code, code_len);
        let deployment_result = self
            .state
            .borrow()
            .deploy_returns
            .get(&(code.to_vec(), Some(salt)))
            .cloned();
        if deployment_result.is_none() {
            return;
        }
        if let Some(Ok(addr)) = deployment_result {
            let contract = slice::from_raw_parts_mut(_contract, 20);
            contract.copy_from_slice(addr.as_ref());
        }
    }
}

impl StorageAccess for TestVM {
    unsafe fn storage_cache_bytes32(&self, key: U256, value: B256) {
        self.state.borrow_mut().storage.insert(key, value);
    }

    fn flush_cache(&self, _clear: bool) {}
    fn storage_load_bytes32(&self, key: U256) -> B256 {
        let curr_state = self.state.borrow();
        if let Some(provider) = &curr_state.provider {
            let rt = Runtime::new().expect("Failed to create runtime");
            let addr = curr_state.contract_address;
            let storage = rt
                .block_on(async { provider.get_storage_at(addr, key).await })
                .unwrap_or_default();
            return B256::from(storage);
        }
        curr_state.storage.get(&key).copied().unwrap_or(B256::ZERO)
    }
}

unsafe impl UnsafeCallAccess for TestVM {
    unsafe fn call_contract(
        &self,
        to: *const u8,
        data: *const u8,
        data_len: usize,
        value: *const u8,
        _gas: u64,
        outs_len: &mut usize,
    ) -> u8 {
        let to_addr = Address::from_slice(slice::from_raw_parts(to, 20));
        let data_slice = slice::from_raw_parts(data, data_len);
        let value_slice = slice::from_raw_parts(value, 32);
        let mut value_bytes = [0u8; 32];
        value_bytes.copy_from_slice(value_slice);
        let value_u256 = U256::from_be_bytes(value_bytes);

        match self.perform_mocked_call(to_addr, data_slice.to_vec(), value_u256) {
            Ok(return_data) => {
                *outs_len = return_data.len();
                0 // Success
            }
            Err(revert_data) => {
                *outs_len = revert_data.len();
                1 // Revert
            }
        }
    }
    unsafe fn delegate_call_contract(
        &self,
        to: *const u8,
        data: *const u8,
        data_len: usize,
        _gas: u64,
        outs_len: &mut usize,
    ) -> u8 {
        let to_addr = Address::from_slice(slice::from_raw_parts(to, 20));
        let data_slice = slice::from_raw_parts(data, data_len);

        match self.perform_mocked_delegate_call(to_addr, data_slice.to_vec()) {
            Ok(return_data) => {
                *outs_len = return_data.len();
                0 // Success
            }
            Err(revert_data) => {
                *outs_len = revert_data.len();
                1 // Revert
            }
        }
    }
    unsafe fn static_call_contract(
        &self,
        to: *const u8,
        data: *const u8,
        data_len: usize,
        _gas: u64,
        outs_len: &mut usize,
    ) -> u8 {
        let to_addr = Address::from_slice(slice::from_raw_parts(to, 20));
        let data_slice = slice::from_raw_parts(data, data_len);

        match self.perform_mocked_static_call(to_addr, data_slice.to_vec()) {
            Ok(return_data) => {
                *outs_len = return_data.len();
                0 // Success
            }
            Err(revert_data) => {
                *outs_len = revert_data.len();
                1 // Revert
            }
        }
    }
}

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
        if let Some(origin) = self.state.borrow().tx_origin {
            return origin;
        }
        self.msg_sender()
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

#[cfg(all(test, not(feature = "reentrant")))]
mod tests {
    use super::*;

    #[test]
    fn test_basic_vm_operations() {
        let vm = TestVM::new();

        vm.set_block_number(12345);
        assert_eq!(vm.block_number(), 12345);

        vm.set_block_timestamp(10);
        assert_eq!(vm.block_timestamp(), 10);

        let sender = Address::from([2u8; 20]);
        vm.set_sender(sender);
        assert_eq!(vm.msg_sender(), sender);
        vm.set_tx_origin(sender);
        assert_eq!(vm.tx_origin(), sender);

        let balance = U256::from(1000);
        vm.set_balance(sender, balance);
        assert_eq!(vm.balance(sender), balance);

        let contract = Address::from([3u8; 20]);
        vm.set_contract_address(contract);
        assert_eq!(vm.contract_address(), contract);

        let code = vec![1u8, 2u8, 3u8];
        vm.set_code(contract, code.clone());
        assert_eq!(vm.code(contract), code);

        let gas_left = 5;
        vm.set_gas_left(gas_left);
        assert_eq!(vm.evm_gas_left(), gas_left);

        let ink_left = 6;
        vm.set_ink_left(ink_left);
        assert_eq!(vm.evm_ink_left(), ink_left);

        let chain_id = 777;
        vm.set_chain_id(chain_id);
        assert_eq!(vm.chain_id(), chain_id);

        let key = U256::from(1);
        let value = B256::new([1u8; 32]);
        vm.set_storage(key, value);
        assert_eq!(vm.get_storage(key), value);

        vm.clear_storage();

        assert_eq!(vm.get_storage(key), B256::ZERO);

        let value = U256::from(2);
        vm.set_value(value);
        assert_eq!(vm.msg_value(), value);
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
}
