// Copyright 2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy_sol_types::sol;

use stylus_proc::SolidityError;

sol! {
    error InsufficientBalance(address from, uint256 have, uint256 want);
    error InsufficientAllowance(address owner, address spender, uint256 have, uint256 want);
}

#[derive(SolidityError)]
enum MyError {
    Unit,
    Two(InsufficientBalance, InsufficientAllowance),
    Named { balance: InsufficientBalance },
}

fn main() {}
