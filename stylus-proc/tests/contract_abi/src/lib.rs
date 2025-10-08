#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloy_sol_types::sol;
use stylus_sdk::{alloy_primitives::U256, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct Counter {
        uint256 number;
    }
}

sol! {
    #[derive(AbiType)]
    struct MixedInput {
        string  textin;
        uint256 numberin;
    }

    #[derive(AbiType)]
    struct MixedResult {
        uint256 number;
        string  text;
        uint256 number2;
    }
}

#[public]
impl Counter {
    #[constructor]
    #[payable]
    pub fn constructor(&mut self, initial_number: U256, _b: String) {
        self.number.set(initial_number);
    }

    pub fn numbers(&self) -> (U256, U256) {
        (self.number.get(), U256::from(0))
    }

    pub fn mixed(&mut self, input: MixedInput) {
        self.number.set(input.numberin);
    }

    pub fn mixed_result(&self) -> MixedResult {
        MixedResult {
            number: self.number.get(),
            text: "hello".to_string(),
            number2: U256::from(4),
        }
    }
}
