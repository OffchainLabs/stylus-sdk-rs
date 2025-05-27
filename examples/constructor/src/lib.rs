// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;


use alloy_sol_types::sol;
use alloy_primitives::{Address, U256};
use stylus_sdk::prelude::*;

sol! {
    error Unauthorized();
}

sol_storage! {
    #[entrypoint]
    pub struct Contract {
        address owner;
        uint256 value;
    }
}

#[derive(SolidityError)]
pub enum ContractErrors {
    Unauthorized(Unauthorized),
}

#[public]
impl Contract {
    /// The constructor sets the owner as the EOA that deployed the contract.
    #[constructor]
    pub fn constructor(&mut self, initial_value: U256) {
        // Use tx_origin instead of msg_sender because we use a factory contract in deployment.
        let owner = self.vm().tx_origin();
        self.owner.set(owner);
        self.value.set(initial_value);
    }

    /// Only the owner can set the value in the contract.
    pub fn set_value(&mut self, value: U256) -> Result<(), ContractErrors> {
        if self.owner.get() != self.vm().msg_sender() {
            return Err(ContractErrors::Unauthorized(Unauthorized{}));
        }
        self.value.set(value);
        Ok(())
    }

    pub fn value(&self) -> U256 {
        self.value.get()
    }

    pub fn owner(&self) -> Address {
        self.owner.get()
    }
}
