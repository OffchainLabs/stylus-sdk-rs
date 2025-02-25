// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Integration test for the `#[public]` macro using composition based inheritance.
//!
//! Currently this simply checks that a contract using this macro can compile successfully.

extern crate alloc;

use stylus_sdk::{
    alloy_primitives::{Address, U256},
    prelude::*,
    storage::{StorageAddress, StorageMap, StorageU256},
};

#[storage]
#[entrypoint]
struct Contract {
    erc20: Erc20,
    ownable: Ownable,
}

#[public]
#[implements(IErc20, IOwnable)]
impl Contract {}

#[storage]
struct Erc20 {
    balances: StorageMap<Address, StorageU256>,
    total_supply: StorageU256,
}

trait IErc20 {
    fn name(&self) -> String;
    fn symbol(&self) -> String;
    fn decimals(&self) -> U256;
    fn total_supply(&self) -> U256;
    fn balance_of(&self, _account: Address) -> U256;
    fn transfer(&mut self, _to: Address, _value: U256) -> bool;
    // fn transfer_from(&mut self, from: Address, to: Address, value: U256) -> bool;
    // fn approve(&mut self, spender: Address, value: U256) -> bool;
    // fn allowance(&self, owner: Address, spender: Address) -> U256;
}

#[public]
impl IErc20 for Contract {
    fn name(&self) -> String {
        todo!()
    }
    fn symbol(&self) -> String {
        todo!()
    }
    fn decimals(&self) -> U256 {
        todo!()
    }
    fn total_supply(&self) -> U256 {
        todo!()
    }
    fn balance_of(&self, _account: Address) -> U256 {
        todo!()
    }
    fn transfer(&mut self, _to: Address, _value: U256) -> bool {
        todo!()
    }
}

#[storage]
struct Ownable {
    owner: StorageAddress,
}

trait IOwnable {
    fn owner(&self) -> Address;
    fn transfer_ownership(&mut self, new_owner: Address) -> bool;
    fn renounce_ownership(&mut self) -> bool;
}

#[public]
impl IOwnable for Contract {
    fn owner(&self) -> Address {
        todo!()
    }
    fn transfer_ownership(&mut self, new_owner: Address) -> bool {
        todo!()
    }
    fn renounce_ownership(&mut self) -> bool {
        todo!()
    }
}
