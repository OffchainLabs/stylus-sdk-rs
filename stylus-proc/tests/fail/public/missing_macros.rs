// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

extern crate alloc;

use stylus_sdk::{
    alloy_primitives::{Address, U256},
    prelude::*,
};

#[storage]
#[entrypoint]
pub struct Contract {}

#[public]
#[implements(Trait1)]
impl Contract {}

// Missing #[public] attribute on trait methods
pub trait Trait1 {
    fn trait1_fn1(&self, _input: U256) -> U256;
}

#[public]
impl Trait1 for Contract {
    fn trait1_fn1(&self, _input: U256) -> U256 {
        todo!()
    }
}

#[public]
pub trait Trait2 {
    fn trait2_fn1(&self, _input: U256) -> U256;
    fn trait2_fn2(&mut self, _to: Address, _value: U256) -> bool;
}

#[public]
impl Trait2 for Contract {
    fn trait2_fn1(&self, _input: U256) -> U256 {
        todo!()
    }

    fn trait2_fn2(&mut self, _to: Address, _value: U256) -> bool {
        todo!()
    }
}

#[public]
pub trait Trait3 {
    fn trait3_fn1(&self, _input: U256) -> U256;
}

// Missing #[public] attribute on impl block
impl Trait3 for Contract {
    fn trait3_fn1(&self, _input: U256) -> U256 {
        todo!()
    }
}

fn main() {}
