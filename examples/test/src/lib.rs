// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;

use stylus_sdk::alloy_sol_types::sol;
/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{
    alloy_primitives::{Address, U256},
    evm,
    prelude::*,
    console,
};
// Define some persistent storage using the Solidity ABI.
// `Counter` will be the entrypoint.
sol_storage! {
    #[entrypoint]
    pub struct Counter {
        uint256 number;
        address owner;
        uint256 last_updated;
    }
}

// Define events
sol! {
event CounterUpdated(address indexed user, uint256 prev_value, uint256 new_value);
}

/// Declare that `Counter` is a contract with the following external methods.
#[public]
impl Counter {
    pub fn calling_console_doesnt_panic_in_test(&self) {
        console!("console called successfully");
    }
    pub fn owner(&self) -> Address {
        self.owner.get()
    }
    pub fn number(&self) -> U256 {
        self.number.get()
    }
    pub fn last_updated(&self) -> U256 {
        self.last_updated.get()
    }
    /// Sets a number in storage to a user-specified value.
    pub fn set_number(&mut self, new_number: U256) {
        let prev = self.number.get();
        self.number.set(new_number);
        // Update the last updated timestamp.
        self.last_updated
            .set(U256::from(self.vm().block_timestamp()));
        // Emit an event
        evm::log(
            self.vm(),
            CounterUpdated {
                user: self.vm().msg_sender(),
                prev_value: prev,
                new_value: self.number.get(),
            },
        );
    }
    /// Sets a number in storage to a user-specified value.
    pub fn mul_number(&mut self, new_number: U256) {
        self.number.set(new_number * self.number.get());
        let prev = self.number.get();
        // Update the last updated timestamp.
        self.last_updated
            .set(U256::from(self.vm().block_timestamp()));
        // Emit an event
        evm::log(
            self.vm(),
            CounterUpdated {
                user: self.vm().msg_sender(),
                prev_value: prev,
                new_value: self.number.get(),
            },
        );
    }
    /// Sets a number in storage to a user-specified value.
    pub fn add_number(&mut self, new_number: U256) {
        self.number.set(new_number + self.number.get());
        let prev = self.number.get();
        // Update the last updated timestamp.
        self.last_updated
            .set(U256::from(self.vm().block_timestamp()));
        // Emit an event
        evm::log(
            self.vm(),
            CounterUpdated {
                user: self.vm().msg_sender(),
                prev_value: prev,
                new_value: self.number.get(),
            },
        );
    }
    /// Increments `number` and updates its value in storage.
    pub fn increment(&mut self) {
        // Increment the number in storage.
        let prev = self.number.get();
        self.set_number(prev + U256::from(1));
        // Update the last updated timestamp.
        self.last_updated
            .set(U256::from(self.vm().block_timestamp()));
        // Emit an event
        evm::log(
            self.vm(),
            CounterUpdated {
                user: self.vm().msg_sender(),
                prev_value: prev,
                new_value: self.number.get(),
            },
        );
    }
    /// Decrements `number` and updates its value in storage.
    /// Returns an error if the number is already zero.
    pub fn decrement(&mut self) -> Result<(), Vec<u8>> {
        let prev = self.number.get();
        if prev == U256::ZERO {
            return Err(b"Counter cannot go below zero".to_vec());
        }

        self.number.set(prev - U256::from(1));
        // Update the last updated timestamp.
        self.last_updated
            .set(U256::from(self.vm().block_timestamp()));
        // Emit an event
        evm::log(
            self.vm(),
            CounterUpdated {
                user: self.vm().msg_sender(),
                prev_value: prev,
                new_value: self.number.get(),
            },
        );

        Ok(())
    }
    /// Adds the wei value from msg_value to the number in storage.
    #[payable]
    pub fn add_from_msg_value(&mut self) {
        let prev = self.number.get();
        self.set_number(prev + self.vm().msg_value());
        // Update the last updated timestamp.
        self.last_updated
            .set(U256::from(self.vm().block_timestamp()));
        // Emit an event
        evm::log(
            self.vm(),
            CounterUpdated {
                user: self.vm().msg_sender(),
                prev_value: prev,
                new_value: self.number.get(),
            },
        );
    }
    // External call example
    pub fn call_external_contract(
        &mut self,
        target: Address,
        data: Vec<u8>,
    ) -> Result<Vec<u8>, Vec<u8>> {
        if self.owner.get() != self.vm().msg_sender() {
            return Err(b"Only owner can call this function".to_vec());
        }
        let context = Call::new_mutating(self);
        let return_data = call(self.vm(), context, target, &data)
            .map_err(|err| format!("{:?}", err).as_bytes().to_vec())?;
        Ok(return_data)
    }
    /// Transfers ownership of the contract to a new address.
    pub fn transfer_ownership(&mut self, new_owner: Address) -> Result<(), Vec<u8>> {
        if self.owner.get() == Address::ZERO {
            self.owner.set(new_owner);
            return Ok(());
        }
        // Check if the owner is already set.
        if self.owner.get() != self.vm().msg_sender() {
            return Err(b"Only owner can call this function".to_vec());
        }
        if new_owner == Address::ZERO {
            return Err(b"Cannot transfer to zero address".to_vec());
        }
        self.owner.set(new_owner);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_console() {
        use stylus_sdk::testing::*;
        let vm = TestVM::default();
        let contract = Counter::from(&vm);
        contract.calling_console_doesnt_panic_in_test();
    }

    #[test]
    fn test_counter() {
        use stylus_sdk::testing::*;
        let vm = TestVM::default();
        let mut contract = Counter::from(&vm);

        assert_eq!(U256::ZERO, contract.number());

        contract.increment();
        assert_eq!(U256::from(1), contract.number());

        contract.add_number(U256::from(3));
        assert_eq!(U256::from(4), contract.number());

        contract.mul_number(U256::from(2));
        assert_eq!(U256::from(8), contract.number());

        contract.set_number(U256::from(100));
        assert_eq!(U256::from(100), contract.number());

        // Override the msg value for future contract method invocations.
        vm.set_value(U256::from(2));

        contract.add_from_msg_value();
        assert_eq!(U256::from(102), contract.number());
    }

    #[test]
    fn test_decrement() {
        use stylus_sdk::testing::*;
        let vm = TestVM::new();
        let mut contract = Counter::from(&vm);

        contract.set_number(U256::from(5));
        // Decrement should succeed
        assert!(contract.decrement().is_ok());
        assert_eq!(contract.number(), U256::from(4));

        // Multiple decrements
        assert!(contract.decrement().is_ok());
        assert_eq!(contract.number(), U256::from(3));

        // Set to zero and try to decrement again
        contract.set_number(U256::ZERO);
        assert!(contract.decrement().is_err());
    }

    #[test]
    fn test_logs() {
        use alloy_primitives::hex;
        use stylus_sdk::alloy_primitives::B256;
        use stylus_sdk::testing::*;
        let vm = TestVM::new();
        let sender = vm.msg_sender();
        let mut contract = Counter::from(&vm);
        // Perform an action that emits an event
        contract.increment();

        // Get the emitted logs
        let logs = vm.get_emitted_logs();
        assert_eq!(logs.len(), 2);

        // Check the event topic (first topic is the event signature)
        // Precalculated the event signature for the event CounterUpdated(address indexed user, uint256 prev_value, uint256 new_value);
        let event_signature: B256 =
            hex!("c9d64952459b33e1dd10d284fe1e9336b8c514cbf51792a888ee7615ca3225d9").into();
        assert_eq!(logs[0].0[0], event_signature);
        // Check that the indexed user address is in the topics
        let user_topic = logs[0].0[1];
        let user_bytes: [u8; 32] = user_topic.into();

        // The indexed address is padded to 32 bytes, extract the last 20 bytes
        let mut user_address = [0u8; 20];
        user_address.copy_from_slice(&user_bytes[12..32]);
        assert_eq!(Address::from(user_address), sender);
    }
    #[test]
    fn test_external_call() {
        use stylus_sdk::testing::*;
        let vm = TestVM::new();
        let mut contract = Counter::from(&vm);
        let sender = vm.msg_sender();
        assert!(contract.transfer_ownership(sender).is_ok());
        // 2) Prepare inputs
        let target = Address::from([0x05; 20]);
        let call_data = vec![1, 2, 3, 4];
        let success_ret = vec![5, 6, 7, 8];
        let error_ret = vec![9, 9, 9];

        // 3) Mock a successful external call
        vm.mock_call(
            target,
            call_data.clone(),
            U256::ZERO,
            Ok(success_ret.clone()),
        );
        let got = contract.call_external_contract(target, call_data.clone());
        assert_eq!(got, Ok(success_ret));

        // 4) Mock a reverting external call
        vm.mock_call(
            target,
            call_data.clone(),
            U256::ZERO,
            Err(error_ret.clone()),
        );
        let err = contract
            .call_external_contract(target, call_data.clone())
            .unwrap_err();
        let expected = format!("Revert({:?})", error_ret).as_bytes().to_vec();
        assert_eq!(err, expected);
    }

    #[test]
    fn test_storage_direct_access() {
        use stylus_sdk::alloy_primitives::{B256, U256};
        use stylus_sdk::testing::*;

        // 1) Create the VM and your Counter instance
        let vm = TestVM::new();
        let mut contract = Counter::from(&vm);

        // 2) Initialize slot 0 to 42 via your setter
        contract.set_number(U256::from(42u64));

        // 2) Storage slot for `count` is the first field → slot 0
        let slot = U256::ZERO;

        // 3) Read it directly — should reflect the constructor value (42)
        let raw = vm.storage_load_bytes32(slot);
        assert_eq!(
            raw,
            B256::from_slice(&U256::from(42u64).to_be_bytes::<32>())
        );

        // 4) Overwrite the slot in the cache, then flush it
        let new_val = U256::from(100u64);
        unsafe {
            vm.storage_cache_bytes32(slot, B256::from_slice(&new_val.to_be_bytes::<32>()));
        }
        vm.flush_cache(false);

        // 5) Now your getter should see the updated value
        assert_eq!(contract.number(), new_val);
    }

    #[test]
    fn test_block_data() {
        use alloy_primitives::{Address, U256};
        use stylus_sdk::testing::*;

        let vm: TestVM = TestVMBuilder::new()
            .sender(Address::from([1u8; 20]))
            .contract_address(Address::from([2u8; 20]))
            .value(U256::from(10))
            .build();

        let mut contract = Counter::from(&vm);

        // 2) Set initial block timestamp & number on the VM
        vm.set_block_timestamp(1_234_567_890);
        vm.set_block_number(100);
        // Increment to trigger timestamp update
        contract.increment();

        // 4) First increment: just call it, then check `last_updated`
        contract.increment();
        assert_eq!(
            contract.last_updated(),
            U256::from(1_234_567_890u64),
            "after first increment, timestamp should be the initial VM timestamp"
        );

        // Update block number and timestamp
        vm.set_block_timestamp(2000000000);
        vm.set_block_number(200);

        // 6) Second increment: call again, then check updated timestamp
        contract.increment();
        assert_eq!(
            contract.last_updated(),
            U256::from(2_000_000_000u64),
            "after second increment, timestamp should reflect VM update"
        );
    }
    #[test]
    fn test_ownership() {
        use stylus_sdk::testing::*;
        // 1) Create the VM and the contract instance
        let vm = TestVM::new();
        let sender = vm.msg_sender();
        let mut contract = Counter::from(&vm);
        let target = Address::from([0x05; 20]);
        let call_data = vec![1, 2, 3, 4];
        let success_ret = vec![5, 6, 7, 8];
        let error_ret = vec![9, 9, 9];
        // 2) Set the contract owner to the sender
        assert!(contract.transfer_ownership(sender).is_ok());

        // Change sender to non-owner
        let non_owner = Address::from([3u8; 20]);
        vm.set_sender(non_owner);

        // Check if non-owner can call external call function before changing ownership
        vm.mock_call(
            target,
            call_data.clone(),
            U256::ZERO,
            Err(error_ret.clone()),
        );
        assert!(contract
            .call_external_contract(target, call_data.clone())
            .is_err());

        // Non-owner should not be able to transfer ownership
        assert!(contract.transfer_ownership(non_owner).is_err());

        // Change back to owner
        vm.set_sender(sender);

        // Owner should be able to transfer ownership
        assert!(contract.transfer_ownership(non_owner).is_ok());
        assert_eq!(contract.owner(), non_owner);
        // Check if non-owner can call external call function after changing ownership
        vm.set_sender(non_owner);
        vm.mock_call(
            target,
            call_data.clone(),
            U256::ZERO,
            Ok(success_ret.clone()),
        );
        let got = contract.call_external_contract(target, call_data.clone());
        assert_eq!(got, Ok(success_ret));
    }
}

//Writing your Own Custom TestVM
// A TestVM is a simple struct implemented in the stylus-test crate that implements the Host trait from stylus_core::host::Host. Anyone can implement the trait and allow for rich testing experiences for Stylus contracts. The TestVM is not the only way to unit test your projects, as you can extend or implement your own.

// Here’s a “general-purpose” extension to TestVM that is just a way to track how many times someone has called into mock_call, so you can assert on how many external calls you set up:
// This is a simple example, but you can imagine more complex scenarios where you might want to track how many times a function was called, or what the arguments were, etc. You can also use this to set up more complex test cases where you need to mock multiple calls with different arguments.

#[cfg(test)]
mod custom_vm_tests {
    use super::*;
    use alloy_primitives::Address;
    use stylus_sdk::testing::TestVM;

    /// A thin wrapper around TestVM that counts how many times
    /// `mock_call` has been invoked.
    pub struct CustomVM {
        inner: TestVM,
        mock_call_count: usize,
    }

    impl CustomVM {
        /// Start with the default TestVM
        pub fn new() -> Self {
            Self {
                inner: TestVM::default(),
                mock_call_count: 0,
            }
        }
        /// **Wrapped** mock_call: increments our counter, then delegates.
        pub fn mock_call(&mut self, target: Address, data: Vec<u8>, ret: Result<Vec<u8>, Vec<u8>>) {
            self.mock_call_count += 1;
            self.inner.mock_call(target, data, U256::ZERO, ret);
        }

        /// New helper: how many mocks have been defined so far?
        pub fn mock_call_count(&self) -> usize {
            self.mock_call_count
        }

        /// Expose the raw TestVM when you need it for `Counter::from(&…)`
        pub fn inner(&self) -> &TestVM {
            &self.inner
        }
    }

    #[test]
    fn test_tracking_number_of_mocks() {
        // 1) Build our custom VM wrapper
        let mut vm = CustomVM::new();

        // 2) Deploy a Counter (or any contract) against the inner TestVM
        let mut contract = Counter::from(vm.inner());

        // 3) Before any mocks, count should be zero
        assert_eq!(vm.mock_call_count(), 0);

        // 4) Define two mock calls
        let addr = Address::from([0x05; 20]);
        let data = vec![1, 2, 3];
        vm.mock_call(addr, data.clone(), Ok(vec![0xAA]));
        vm.mock_call(addr, data, Err(vec![0xBB]));

        // 5) Now our helper sees exactly two mocks
        assert_eq!(vm.mock_call_count(), 2);

        // 6) And of course external calls still work through inner:
        let _ = contract.call_external_contract(addr, vec![1, 2, 3]);
        let _ = contract.call_external_contract(addr, vec![1, 2, 3]);

        // 7) But the number of *defined* mocks remains the same
        assert_eq!(vm.mock_call_count(), 2);
    }
}
