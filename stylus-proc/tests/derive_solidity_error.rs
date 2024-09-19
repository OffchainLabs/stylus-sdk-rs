// Copyright 2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy_sol_types::sol;

use stylus_proc::{public, SolidityError};

extern crate alloc;

sol! {
    error InsufficientBalance(address from, uint256 have, uint256 want);
    error InsufficientAllowance(address owner, address spender, uint256 have, uint256 want);
}

#[derive(SolidityError)]
pub enum Erc20Error {
    InsufficientBalance(InsufficientBalance),
    InsufficientAllowance(InsufficientAllowance),
}

struct Contract {}

#[public]
impl Contract {
    pub fn fallible_method() -> Result<(), Erc20Error> {
        Ok(())
    }
}
