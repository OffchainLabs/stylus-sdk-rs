// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Integration test for the `#[public]` macro using composition based inheritance.
//!
//! Currently this simply checks that a contract using this macro can compile successfully.

#![allow(dead_code)]
#![allow(unused_variables)]

extern crate alloc;

use stylus_sdk::prelude::*;

#[storage]
#[entrypoint]
struct Contract {}

#[public]
#[implements(MyTrait<u32, u32, Output = u32>)]
impl Contract {}

trait MyTrait<Input1, Input2> {
    type Output;
    fn foo(&self, input1: Input1, input2: Input2) -> Self::Output;
}

#[public]
impl MyTrait<u32, u32> for Contract {
    type Output = u32;
    fn foo(&self, input1: u32, input2: u32) -> Self::Output {
        input1 + input2
    }
}
