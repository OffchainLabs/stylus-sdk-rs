#![cfg_attr(not(feature = "export-abi"), no_std, no_main)]
extern crate alloc;

use stylus_sdk::{alloy_primitives::{Address, U256}, prelude::*};
use alloc::{vec, vec::Vec};

sol_storage! {
    #[entrypoint]
    pub struct Caller {
        address target;
        uint256 last_result;
    }
}

#[public]
impl Caller {
    pub fn set_target(&mut self, target: Address) {
        self.target.set(target);
    }

    pub fn get_target(&self) -> Address {
        self.target.get()
    }
    
    pub fn set_last_result(&mut self, value: U256) {
        self.last_result.set(value);
    }
    
    pub fn get_last_result(&self) -> U256 {
        self.last_result.get()
    }
}