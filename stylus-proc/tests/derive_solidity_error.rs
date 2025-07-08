// Copyright 2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![allow(dead_code)]

// contract-client-gen feature can generate code that makes some imports of this file unused
#![allow(unused_imports)]

extern crate alloc;

use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;

use stylus_proc::{public, SolidityError};
use stylus_sdk::prelude::*;

sol! {
    error InsufficientBalance(address from, uint256 have, uint256 want);
    error InsufficientAllowance(address owner, address spender, uint256 have, uint256 want);
}

#[derive(SolidityError)]
pub enum Erc20Error {
    InsufficientBalance(InsufficientBalance),
    InsufficientAllowance(InsufficientAllowance),
}

#[storage]
#[entrypoint]
struct Contract {}

#[public]
impl Contract {
    /// Test using the defined error in a result value
    pub fn fallible_method() -> Result<(), Erc20Error> {
        Err(InsufficientBalance {
            from: Address::ZERO,
            have: U256::ZERO,
            want: U256::ZERO,
        }
        .into())
    }
}

#[cfg(all(not(feature = "contract-client-gen"), feature = "trybuild-tests"))]
#[test]
fn test_derive_solidity_error_failures() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/derive_solidity_error/invalid_variants.rs");
}
