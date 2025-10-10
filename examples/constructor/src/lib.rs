// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(feature = "contract-client-gen", allow(unused_imports))]

extern crate alloc;

use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;
use stylus_sdk::prelude::*;

sol! {
    error Unauthorized();
}

sol_storage! {
    #[entrypoint]
    pub struct Contract {
        address owner;
        uint256 number;
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
    #[payable]
    pub fn constructor(&mut self, initial_number: U256) {
        // Use tx_origin instead of msg_sender because we use a factory contract in deployment.
        let owner = self.vm().tx_origin();
        self.owner.set(owner);
        self.number.set(initial_number);
    }

    /// Only the owner can set the number in the contract.
    pub fn set_number(&mut self, number: U256) -> Result<(), ContractErrors> {
        if self.owner.get() != self.vm().msg_sender() {
            return Err(ContractErrors::Unauthorized(Unauthorized {}));
        }
        self.number.set(number);
        Ok(())
    }

    pub fn number(&self) -> U256 {
        self.number.get()
    }

    pub fn owner(&self) -> Address {
        self.owner.get()
    }
}
