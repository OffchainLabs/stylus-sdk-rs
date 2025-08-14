// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Integration test for using call contexts with sol_interface macros to generate
//! cross-contract call bindings.
//!
//! Currently this simply checks that a contract using this macro can compile successfully.

#![allow(dead_code)]
#![allow(unused_variables)]
// contract-client-gen feature can generate code that makes some imports of this file unused
#![allow(unused_imports)]

extern crate alloc;

use alloy_primitives::U256;
use stylus_sdk::prelude::*;

sol_interface! {
    interface IFoo {
        function viewFoo() view external;
        function mutateFoo() external;
        function payFoo() payable external;
    }
}

#[storage]
#[entrypoint]
pub struct Contract {}

#[public]
impl Contract {
    pub fn mutate(&mut self, methods: IFoo) -> Result<(), Vec<u8>> {
        let cfg = Call::new_mutating(self);
        methods.mutate_foo(self.vm(), cfg).unwrap();
        Ok(())
    }
    pub fn view(&mut self, methods: IFoo) -> Result<(), Vec<u8>> {
        let cfg = Call::new();
        methods.view_foo(self.vm(), cfg).unwrap();
        Ok(())
    }
    #[payable]
    pub fn pay(&mut self, methods: IFoo) -> Result<(), Vec<u8>> {
        let cfg = Call::new_payable(self, U256::from(1));
        methods.pay_foo(self.vm(), cfg).unwrap();
        Ok(())
    }
}
