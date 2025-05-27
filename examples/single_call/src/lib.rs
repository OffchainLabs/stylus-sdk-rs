// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use stylus_sdk::{abi::Bytes, alloy_primitives::Address, prelude::*};

#[storage]
#[entrypoint]
pub struct SingleCall;

#[public]
impl SingleCall {
    pub fn execute(&self, target: Address, data: Bytes) -> Bytes {
        unsafe {
            let result = RawCall::new(self.vm()).call(target, &data);
            result.unwrap().into()
        }
    }
}
