// Copyright 2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Integration test for the `#[public]` macro
//!
//! Currently this simply checks that a contract using this macro can compile successfully.

extern crate alloc;

use alloy_primitives::U256;
use stylus_proc::public;
use stylus_sdk::storage::StorageU256;

struct Contract {}

#[public]
impl Contract {
    #[payable]
    fn method() {}
}

struct ContractWithConstructor {
    value: StorageU256,
}

#[public]
impl ContractWithConstructor {
    #[constructor]
    fn constructor(&mut self, value: U256) {
        self.value.set(value);
    }

    fn value(&self) -> Result<U256, Vec<u8>> {
        Ok(self.value.get())
    }
}

#[test]
fn test_public_failures() {
    let t = trybuild::TestCases::new();
    #[cfg(not(feature = "export-abi"))]
    t.compile_fail("tests/fail/public/generated.rs");
    t.compile_fail("tests/fail/public/macro_errors.rs");
    t.compile_fail("tests/fail/public/constructor.rs");
}
