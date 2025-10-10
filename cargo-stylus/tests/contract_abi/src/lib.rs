#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloy_sol_types::sol;
use stylus_sdk::{
    alloy_primitives::{Address, FixedBytes, U256},
    prelude::*,
    ArbResult,
};

sol_storage! {
    #[entrypoint]
    pub struct Counter {
        uint256 number;
    }
}

sol! {
    #[derive(AbiType)]
    struct MixedInput {
        string  textin;
        uint256 numberin;
    }

    #[derive(AbiType)]
    struct MixedResult {
        uint256 number;
        string  text;
        uint256 number2;
    }
}

#[public]
#[implements(Trait1<U256, Address, Output = U256>, Trait2)]
impl Counter {
    #[constructor]
    #[payable]
    pub fn constructor(&mut self, initial_number: U256, _b: String) {
        self.number.set(initial_number);
    }
    
    pub fn pure_fn() -> U256 {
        U256::from(1234)
    }
    
    pub fn no_input_no_output(&self) {}

    pub fn numbers(&self) -> (U256, U256) {
        (self.number.get(), U256::from(0))
    }

    pub fn mixed(&mut self, input: MixedInput) {
        self.number.set(input.numberin);
    }

    pub fn mixed_result(&self) -> MixedResult {
        MixedResult {
            number: self.number.get(),
            text: "hello".to_string(),
            number2: U256::from(4),
        }
    }
}

#[public]
pub trait Trait1<Input1, Input2> {
    type Output;
    fn one_input_one_output(&self, input: U256) -> Self::Output;
    fn multiple_inputs_multiple_outputs(
        &self,
        input1: Input1,
        input2: Input2,
    ) -> (U256, bool, Address, FixedBytes<32>);
    fn mutable(&mut self) -> bool;
    fn fails(&self);
}

#[public]
impl Trait1<U256, Address> for Counter {
    type Output = U256;

    fn one_input_one_output(&self, input: U256) -> Self::Output {
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

#[public]
pub trait Trait2 {
    fn outputs_result_ok(&self) -> Result<(U256, U256), Vec<u8>>;
    fn outputs_result_err(&self) -> Result<U256, Vec<u8>>;
    fn outputs_arbresult_ok(&self) -> ArbResult;
    fn outputs_arbresult_err(&self) -> ArbResult;
}

#[public]
impl Trait2 for Counter {
    fn outputs_result_ok(&self) -> Result<(U256, U256), Vec<u8>> {
        Ok((U256::from(1234), U256::from(5678)))
    }

    fn outputs_result_err(&self) -> Result<U256, Vec<u8>> {
        Err(vec![0x01, 0x02, 0x03])
    }

    fn outputs_arbresult_ok(&self) -> ArbResult {
        Ok(Vec::from([33, 34, 35]))
    }

    fn outputs_arbresult_err(&self) -> ArbResult {
        Err(vec![0x01, 0x02, 0x03])
    }
}
