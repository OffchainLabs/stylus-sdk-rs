// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

// Only run this as a WASM if the export-abi feature is not set.
#![cfg_attr(not(any(feature = "export-abi", test)), no_main)]
#![cfg_attr(feature = "contract-client-gen", allow(unused_imports))]

extern crate alloc;

// Modules and imports
pub mod erc20;
pub mod ierc20;

use crate::erc20::{Erc20, Erc20Params};
use crate::ierc20::{Erc20Error, IErc20};
use alloy_primitives::{Address, U256};
use stylus_sdk::prelude::*;

/// Immutable definitions
#[derive(Default)]
pub struct StylusTestTokenParams;
impl Erc20Params for StylusTestTokenParams {
    const NAME: &'static str = "StylusTestToken";
    const SYMBOL: &'static str = "STTK";
    const DECIMALS: u8 = 18;
}

// Define the entrypoint as a Solidity storage object. The sol_storage! macro
// will generate Rust-equivalent structs with all fields mapped to Solidity-equivalent
// storage slots and types.
sol_storage! {
    #[entrypoint]
    struct StylusTestToken {
        Erc20<StylusTestTokenParams> erc20;
    }
}

#[public]
#[implements(IErc20)]
impl StylusTestToken {
    /// Mints tokens
    pub fn mint(&mut self, value: U256) -> Result<(), Erc20Error> {
        self.erc20.mint(self.vm().msg_sender(), value)?;
        Ok(())
    }

    /// Mints tokens to another address
    pub fn mint_to(&mut self, to: Address, value: U256) -> Result<(), Erc20Error> {
        self.erc20.mint(to, value)?;
        Ok(())
    }

    /// Burns tokens
    pub fn burn(&mut self, value: U256) -> Result<(), Erc20Error> {
        self.erc20.burn(self.vm().msg_sender(), value)?;
        Ok(())
    }
}

#[public]
impl IErc20 for StylusTestToken {
    fn name(&self) -> String {
        Erc20::<StylusTestTokenParams>::name()
    }

    fn symbol(&self) -> String {
        Erc20::<StylusTestTokenParams>::symbol()
    }

    fn decimals(&self) -> u8 {
        Erc20::<StylusTestTokenParams>::decimals()
    }

    fn total_supply(&self) -> U256 {
        self.erc20.total_supply()
    }

    fn balance_of(&self, owner: Address) -> U256 {
        self.erc20.balance_of(owner)
    }

    fn transfer(&mut self, to: Address, value: U256) -> Result<bool, Erc20Error> {
        self.erc20.transfer(to, value)
    }

    fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Erc20Error> {
        self.erc20.transfer_from(from, to, value)
    }

    fn approve(&mut self, spender: Address, value: U256) -> bool {
        self.erc20.approve(spender, value)
    }

    fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self.erc20.allowance(owner, spender)
    }
}
