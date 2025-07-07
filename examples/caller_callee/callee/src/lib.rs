// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;

use stylus_sdk::{
    alloy_primitives::{Address, FixedBytes, U256},
    prelude::*,
    ArbResult,
};

#[storage]
#[entrypoint]
pub struct Callee {}

#[public]
#[implements(Trait1, Trait2)]
impl Callee {
    fn no_input_no_output(&self) {}
}

pub trait Trait1 {
    fn one_input_one_output(&self, input: U256) -> U256;
    fn multiple_inputs_multiple_outputs(
        &self,
        input1: U256,
        input2: Address,
    ) -> (U256, bool, Address, FixedBytes<32>);
    fn mutable(&mut self) -> bool;
    fn fails(&self);
}

#[public]
impl Trait1 for Callee {
    fn one_input_one_output(&self, input: U256) -> U256 {
        input.saturating_add(U256::from(1))
    }

    fn multiple_inputs_multiple_outputs(
        &self,
        input1: U256,
        input2: Address,
    ) -> (U256, bool, Address, FixedBytes<32>) {
        let output1 = input1.saturating_add(U256::from(2));
        let output2 = true;
        let output3 = input2;
        let output4 = FixedBytes::from([0x01; 32]);
        (output1, output2, output3, output4)
    }

    fn mutable(&mut self) -> bool {
        true
    }

    fn fails(&self) {
        panic!("This function is designed to fail");
    }
}

pub trait Trait2 {
    fn outputs_result_ok(&self) -> Result<U256, Vec<u8>>;
    fn outputs_result_err(&self) -> Result<U256, Vec<u8>>;
    fn outputs_arbresult_ok(&self) -> ArbResult;
    fn outputs_arbresult_err(&self) -> ArbResult;
}

#[public]
impl Trait2 for Callee {
    fn outputs_result_ok(&self) -> Result<U256, Vec<u8>> {
        Ok(U256::from(1234))
    }

    fn outputs_result_err(&self) -> Result<U256, Vec<u8>> {
        Err(vec![0x01, 0x02, 0x03])
    }

    fn outputs_arbresult_ok(&self) -> ArbResult {
        Ok(vec![0x01, 0x02, 0x03])
    }

    fn outputs_arbresult_err(&self) -> ArbResult {
        Err(vec![0x01, 0x02, 0x03])
    }
}
