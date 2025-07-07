// Copyright 2025, Offchain Labs, Iargsnc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;

use callee::Callee;
use stylus_sdk::{
    alloy_primitives::{Address, FixedBytes, U256},
    console,
    prelude::*,
    ArbResult,
};

#[storage]
#[entrypoint]
pub struct Caller {}

#[public]
impl Caller {
    fn no_input_no_output(&self, callee_addr: Address) {
        let callee = Callee::new(callee_addr);
        callee
            .no_input_no_output(self.vm(), Call::new())
            .expect("Call failed")
    }

    fn one_input_one_output(&self, callee_addr: Address, input: U256) -> U256 {
        let callee = Callee::new(callee_addr);
        callee
            .one_input_one_output(self.vm(), Call::new(), input)
            .expect("Call failed")
    }

    fn multiple_inputs_multiple_outputs(
        &self,
        callee_addr: Address,
        input1: U256,
        input2: Address,
    ) -> (U256, bool, Address, FixedBytes<32>) {
        let callee = Callee::new(callee_addr);
        callee
            .multiple_inputs_multiple_outputs(self.vm(), Call::new(), input1, input2)
            .expect("Call failed")
    }

    fn mutable(&mut self, callee_addr: Address) -> bool {
        let callee = Callee::new(callee_addr);
        let ctx = Call::new_mutating(self);
        callee.mutable(self.vm(), ctx).expect("Call failed")
    }

    fn fails(&self, callee_addr: Address) {
        let callee = Callee::new(callee_addr);
        callee.fails(self.vm(), Call::new()).expect("Call failed")
    }

    // fn outputs_result_ok(&self, callee_addr: Address) -> Result<U256, Vec<u8>> {
    //     let callee = Callee::new(callee_addr);
    //     callee.outputs_result_ok(self.vm(), Call::new())
    // }

    // fn outputs_result_err(&self) -> Result<U256, Vec<u8>> {}
    // fn outputs_arbresult_ok(&self) -> ArbResult {}
    // fn outputs_arbresult_err(&self) -> ArbResult {}
}
