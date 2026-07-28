// Copyright 2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! An example contract that opts into a pinned `wasm-opt` (Binaryen) post-build step.
//!
//! The `[wasm-opt]` table in this crate's `Stylus.toml` enables a reproducible optimization pass
//! (`wasm-opt -Oz`) that is applied at deploy time and replayed identically during
//! `cargo stylus verify`, so the deployed bytes stay reproducibly verifiable (including via
//! Arbiscan managed verification). The contract itself is a simple counter; the integration test
//! in `tests/` deploys and verifies it, then exercises the counter to confirm optimization
//! preserved the contract's behavior.

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(feature = "contract-client-gen", allow(unused_imports))]

extern crate alloc;

use stylus_sdk::{alloy_primitives::U256, prelude::*, storage::StorageU256};

#[storage]
#[entrypoint]
pub struct Counter {
    count: StorageU256,
}

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

    /// Increments count by 1.
    pub fn inc(&mut self) -> Result<(), Vec<u8>> {
        let count = self.count.get() + U256::from(1);
        self.set_count(count)
    }

    /// Decrements count by 1.
    pub fn dec(&mut self) -> Result<(), Vec<u8>> {
        let count = self.count.get() - U256::from(1);
        self.set_count(count)
    }
}
