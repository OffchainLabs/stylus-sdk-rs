// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;

use stylus_sdk::{
    abi::Bytes,
    alloy_primitives::{Address, U256},
    call::{call, static_call, RawCall},
    prelude::*,
};

sol_interface! {
    interface IService {
        function makePayment(address user) payable external returns (string);
        function getConstant() pure external returns (bytes32);
    }

    interface IMethods {
        function pureFoo() external pure;
        function viewFoo() external view;
        function writeFoo() external;
        function payableFoo() external payable;
    }
}

#[storage]
#[entrypoint]
pub struct ExampleContract;
#[public]
impl ExampleContract {
    // Call the other contract passing the calldata
    pub fn execute(&self, target: Address, data: Bytes) -> Bytes {
        unsafe {
            let result = RawCall::new(self.vm()).call(target, &data);
            result.unwrap().into()
        }
    }

    // simple call to contract using interface
    pub fn simple_call(&mut self, account: IService, user: Address) -> Result<String, Vec<u8>> {
        // Calls the make_payment method
        let config = Call::new_mutating(self);
        Ok(account.make_payment(self.vm(), config, user)?)
    }

    #[payable]
    // configuring gas and value with Call
    pub fn call_with_gas_value(
        &mut self,
        account: IService,
        user: Address,
    ) -> Result<String, Vec<u8>> {
        let config = Call::new_payable(self, self.vm().msg_value()) // Use the transferred value
            .gas(self.vm().evm_gas_left() / 2); // Use half the remaining gas
        Ok(account.make_payment(self.vm(), config, user)?)
    }

    pub fn call_pure(&self, methods: IMethods) -> Result<(), Vec<u8>> {
        Ok(methods.pure_foo(self.vm(), Call::new())?) // `pure` methods might lie about not being `view`
    }

    pub fn call_view(&self, methods: IMethods) -> Result<(), Vec<u8>> {
        Ok(methods.view_foo(self.vm(), Call::new())?)
    }

    pub fn call_write(&mut self, methods: IMethods) -> Result<(), Vec<u8>> {
        let config = Call::new();
        methods.view_foo(self.vm(), config)?;
        let config = Call::new_mutating(self);
        Ok(methods.write_foo(self.vm(), config)?)
    }

    #[payable]
    pub fn call_payable(&mut self, methods: IMethods) -> Result<(), Vec<u8>> {
        let config = Call::new_mutating(self);
        methods.write_foo(self.vm(), config)?;
        let config = Call::new_payable(self, U256::ZERO);
        Ok(methods.payable_foo(self.vm(), config)?)
    }

    // When writing Stylus libraries, a type might not be TopLevelStorage and therefore &self or &mut self won’t work. Building a Call from a generic parameter via new_in is the usual solution.
    pub fn make_generic_call<T: TopLevelStorage + core::borrow::Borrow<Self>>(
        storage: &mut T, // This could be `&mut self`, or another type implementing `TopLevelStorage`
        account: IService, // Interface for calling the target contract
        user: Address,
    ) -> Result<String, Vec<u8>> {
        let vm = storage.borrow().vm();
        let msg_value = vm.msg_value(); // Use the transferred value
        let gas = vm.evm_gas_left() / 2; // Use half the remaining gas
        let config = Call::new_payable(storage, msg_value).gas(gas); // Take exclusive access to all contract storage
        Ok(account.make_payment(storage.borrow().vm(), config, user)?) // Call using the configured parameters
    }

    // Low level Call
    pub fn execute_call(
        &mut self,
        contract: Address,
        calldata: Vec<u8>, // Calldata is supplied as a Vec<u8>
    ) -> Result<Vec<u8>, Vec<u8>> {
        let config = Call::new_mutating(self) // Configuration for gas, value, etc.
            .gas(self.vm().evm_gas_left()); // Use half the remaining gas
        let return_data = call(
            // Perform a low-level `call`
            self.vm(),
            config,
            contract,  // The target contract address
            &calldata, // Raw calldata to be sent
        )?;

        // Return the raw return data from the contract call
        Ok(return_data)
    }

    // Low level Static Call
    pub fn execute_static_call(
        &mut self,
        contract: Address,
        calldata: Vec<u8>,
    ) -> Result<Vec<u8>, Vec<u8>> {
        // Perform a low-level `static_call`, which does not modify state
        let return_data = static_call(
            self.vm(),
            Call::new(), // Configuration for the call
            contract,    // Target contract
            &calldata,   // Raw calldata
        )?;

        // Return the raw result data
        Ok(return_data)
    }

    // Using Unsafe RawCall
    pub fn raw_call_example(
        &mut self,
        contract: Address,
        calldata: Vec<u8>,
    ) -> Result<Vec<u8>, Vec<u8>> {
        unsafe {
            let data = RawCall::new_delegate(self.vm())
                .gas(2100) // Set gas to 2100
                .limit_return_data(0, 32) // Limit return data to 32 bytes
                .flush_storage_cache() // flush the storage cache before the call
                .call(contract, &calldata)?; // Execute the call
            Ok(data) // Return the raw result
        }
    }
}
