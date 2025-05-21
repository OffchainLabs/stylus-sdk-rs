// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

// Only run this as a WASM if the export-abi feature is not set.
#![cfg_attr(not(any(feature = "export-abi", test)), no_main)]
extern crate alloc;

// Modules and imports
mod erc20;

use crate::erc20::{Erc20, Erc20Error, Erc20Params};
use alloy_primitives::{Address, U256};
use stylus_sdk::prelude::*;

/// Immutable definitions
struct StylusTestTokenParams;
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
        // Allows erc20 to access StylusTestToken's storage and make calls
        #[borrow]
        Erc20<StylusTestTokenParams> erc20;
    }
}

#[public]
#[inherit(Erc20<StylusTestTokenParams>)]
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
