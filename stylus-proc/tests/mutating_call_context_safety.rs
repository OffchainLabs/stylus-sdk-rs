// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Integration test for using call contexts with sol_interface macros to generate
//! cross-contract call bindings.
//!
//! Currently this simply checks that a contract using this macro can compile successfully.

#![allow(dead_code)]
#![allow(unused_variables)]

extern crate alloc;

use stylus_sdk::prelude::*;

sol_interface! {
    interface IFoo {
        function viewFoo() external pure;
    }
}

#[entrypoint]
#[storage]
struct Contract {}

#[public]
impl Contract {
    pub fn execute(&mut self, methods: IFoo) -> Result<(), Vec<u8>> {
        let cfg = Call::new().gas(1_000_000);
        methods.view_foo(self.vm(), cfg).unwrap();
        Ok(())
    }
}
