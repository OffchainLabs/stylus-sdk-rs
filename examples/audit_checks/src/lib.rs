#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

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
    #[fallback]
    pub fn my_fallback(&mut self, _input: &[u8]) -> Result<Vec<u8>, Vec<u8>> {
        self.number.set(U256::from(1u64));
        Ok(vec![])
    }
    #[fallback]
    pub fn fallo(&mut self, _input: &[u8]) -> Result<Vec<u8>, Vec<u8>> {
        Ok(vec![])
    }
}
