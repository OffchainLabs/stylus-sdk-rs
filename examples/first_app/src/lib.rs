// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(feature = "contract-client-gen", allow(unused_imports))]

extern crate alloc;

use stylus_sdk::{alloy_primitives::U256, prelude::*, storage::StorageU256};

/// The solidity_storage macro allows this struct to be used in persistent
/// storage. It accepts fields that implement the StorageType trait. Built-in
/// storage types for Solidity ABI primitives are found under
/// stylus_sdk::storage.
#[storage]
/// The entrypoint macro defines where Stylus execution begins. External methods
/// are exposed by annotating an impl for this struct with #[external] as seen
/// below.
#[entrypoint]
pub struct Counter {
    count: StorageU256,
}

/// Define an implementation of the Counter struct, defining a set_count
/// as well as inc and dec methods using the features of the Stylus SDK.
#[public]
impl Counter {
    /// Gets the number from storage.
    pub fn get(&self) -> Result<U256, Vec<u8>> {
        Ok(self.count.get())
    }

    /// Sets the count in storage to a user-specified value.
    pub fn set_count(&mut self, count: U256) -> Result<(), Vec<u8>> {
        self.count.set(count);
        Ok(())
    }

    /// Increments count by 1
    pub fn inc(&mut self) -> Result<(), Vec<u8>> {
        let count = self.count.get() + U256::from(1);
        self.set_count(count)
    }

    /// Decrements count by 1
    pub fn dec(&mut self) -> Result<(), Vec<u8>> {
        let count = self.count.get() - U256::from(1);
        self.set_count(count)
    }
}
