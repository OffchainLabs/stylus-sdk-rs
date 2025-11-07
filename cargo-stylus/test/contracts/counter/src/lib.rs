#![cfg_attr(not(feature = "export-abi"), no_std, no_main)]
extern crate alloc;

use alloc::{vec, vec::Vec};
use stylus_sdk::{alloy_primitives::U256, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct Counter {
        uint256 number;
    }
}

#[public]
impl Counter {
    pub fn number(&self) -> U256 {
        self.number.get()
    }

    pub fn set_number(&mut self, new_number: U256) {
        self.number.set(new_number);
    }

    pub fn increment(&mut self) {
        let value = self.number.get();
        self.set_number(value + U256::from(1));
    }

    pub fn add(&mut self, value: U256) {
        let current = self.number.get();
        self.set_number(current + value);
    }
}

