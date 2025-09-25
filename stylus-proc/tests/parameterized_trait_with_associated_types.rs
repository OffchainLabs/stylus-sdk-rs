// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Test for the `#[public]` macro using composition based inheritance and associated types.
//!
//! Currently this simply checks that a contract using this macro can compile successfully.

// For now export-abi doesn't support associated types
#![cfg(not(feature = "export-abi"))]

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

extern crate alloc;

use stylus_sdk::abi::AbiType;
use stylus_sdk::prelude::*;

#[storage]
#[entrypoint]
pub struct Contract {}

#[public]
#[implements(MyTrait<u32, u32, Output = u32>)]
impl Contract {}

#[public]
pub trait MyTrait<Input1, Input2> {
    type Output: AbiType;
    fn foo(&self, input1: Input1, input2: Input2) -> Self::Output;
}

#[public]
impl MyTrait<u32, u32> for Contract {
    type Output = u32;
    fn foo(&self, input1: u32, input2: u32) -> Self::Output {
        input1 + input2
    }
}
