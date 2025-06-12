// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;

use alloc::vec;
use stylus_sdk::alloy_primitives::Address;
use stylus_sdk::prelude::*;
use stylus_sdk::storage::StorageAddress;

use stylus_sdk::alloy_primitives::U256;
use stylus_sdk::console;
use stylus_sdk::storage::StorageU256;

#[storage]
#[entrypoint]
pub struct ExampleContract {
    owner: StorageAddress,
    data: StorageU256,
}

#[public]
impl ExampleContract {
    // External function to set the data
    pub fn set_data(&mut self, value: U256) {
        self.data.set(value);
    }

    // External function to get the data
    pub fn get_data(&self) -> U256 {
        self.data.get()
    }

    // External function to get the contract owner
    pub fn get_owner(&self) -> Address {
        self.owner.get()
    }
}

impl ExampleContract {
    // Internal function to set a new owner
    pub fn set_owner(&mut self, new_owner: Address) {
        self.owner.set(new_owner);
    }

    // Internal function to log data
    pub fn log_data(&self) {
        let _data = self.data.get();
        console!("Current data is: {:?}", _data);
    }
}
