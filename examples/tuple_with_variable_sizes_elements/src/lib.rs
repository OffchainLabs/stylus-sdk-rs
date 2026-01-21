// Copyright 2025, Offchain Labs, Iargsnc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;

use stylus_sdk::{alloy_primitives::U256, prelude::*};

#[storage]
#[entrypoint]
pub struct TupleWithVariableSizesElements {}

#[public]
impl TupleWithVariableSizesElements {
    fn only_string(&self) -> String {
        "hello".into()
    }

    fn only_vec(&self) -> Vec<u8> {
        vec![1, 2, 3, 4, 5]
    }

    fn u256_and_u256(&self) -> (U256, U256) {
        (U256::from(1), U256::from(2))
    }

    fn u256_and_string(&self) -> (U256, String) {
        (U256::from(42), "world".into())
    }

    fn u256_and_vec(&self) -> (U256, Vec<u8>) {
        (U256::from(100), vec![6, 7, 8, 9, 10])
    }
}
