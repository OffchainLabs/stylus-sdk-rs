// Copyright 2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Integration test for the `#[public]` macro
//!
//! Currently this simply checks that a contract using this macro can compile successfully.

extern crate alloc;

use stylus_proc::public;

struct Contract {}

#[public]
impl Contract {
    #[payable]
    fn method() {}
}

#[test]
fn test_public_failures() {
    let t = trybuild::TestCases::new();
    #[cfg(not(feature = "export-abi"))]
    t.compile_fail("tests/fail/public/generated.rs");
    t.compile_fail("tests/fail/public/macro_errors.rs");
}
