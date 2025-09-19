// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;
use stylus_sdk::prelude::*;

// Declare Solidity error types
sol! {
    error InsufficientBalance(address from, uint256 have, uint256 want);
    error InsufficientAllowance(address owner, address spender, uint256 have, uint256 want);
}

/// Represents the ways methods may fail.
#[derive(SolidityError)]
pub enum Erc20Error {
    InsufficientBalance(InsufficientBalance),
    InsufficientAllowance(InsufficientAllowance),
}

/// Trait that contains the Erc20 token methods.
#[public]
pub trait IErc20 {
    /// Immutable token name
    fn name(&self) -> String;

    /// Immutable token symbol
    fn symbol(&self) -> String;

    /// Immutable token decimals
    fn decimals(&self) -> u8;

    /// Total supply of tokens
    fn total_supply(&self) -> U256;

    /// Balance of `address`
    fn balance_of(&self, owner: Address) -> U256;

    /// Transfers `value` tokens from msg::sender() to `to`
    fn transfer(&mut self, to: Address, value: U256) -> Result<bool, Erc20Error>;

    /// Transfers `value` tokens from `from` to `to`
    /// (msg::sender() must be able to spend at least `value` tokens from `from`)
    fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Erc20Error>;

    /// Approves the spenditure of `value` tokens of msg::sender() to `spender`
    fn approve(&mut self, spender: Address, value: U256) -> bool;

    /// Returns the allowance of `spender` on `owner`'s tokens
    fn allowance(&self, owner: Address, spender: Address) -> U256;
}
