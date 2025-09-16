// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;

use alloy_primitives::{Bytes, U256};
use stylus_sdk::prelude::*;

sol_storage! {
    #[entrypoint]
    pub struct Tuples {
    }
}

#[public]
impl Tuples {
    pub fn numbers(&mut self) -> (U256, U256, U256) {
        (U256::from(100), U256::from(200), U256::from(300))
    }

    pub fn bytes_and_number(&mut self) -> (Bytes, U256) {
        (Bytes::from(vec![1, 2, 3]), U256::from(42))
    }
}
