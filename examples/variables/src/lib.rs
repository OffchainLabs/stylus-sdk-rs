// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;

use alloy_primitives::{U16, U256};
use stylus_sdk::{
    console,
    prelude::*,
    storage::{StorageAddress, StorageBool, StorageU256},
};

#[storage]
#[entrypoint]
pub struct Contract {
    initialized: StorageBool,
    owner: StorageAddress,
    max_supply: StorageU256,
}

#[public]
impl Contract {
    // State variables are initialized in an `init` function.
    pub fn init(&mut self) -> Result<(), Vec<u8>> {
        // We check if contract has been initialized before.
        // We return if so, we initialize if not.
        let initialized = self.initialized.get();
        if initialized {
            return Ok(());
        }
        self.initialized.set(true);

        // We set the contract owner to the caller,
        // which we get from the global msg module
        self.owner.set(self.vm().msg_sender());
        self.max_supply.set(U256::from(10_000));

        Ok(())
    }

    pub fn do_something(&self) -> Result<(), Vec<u8>> {
        // Local variables are not saved to the blockchain
        // 16-bit Rust integer
        let _i = 456_u16;
        // 16-bit int inferred from U16 Alloy primitive
        let _j = U16::from(123);

        // Here are some global variables
        let _timestamp = self.vm().block_timestamp();
        let _amount = self.vm().msg_value();

        console!("Local variables: {_i}, {_j}");
        console!("Global variables: {_timestamp}, {_amount}");

        Ok(())
    }
}
