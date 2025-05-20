// Only run this as a WASM if the export-abi feature is not set.
#![cfg_attr(not(any(feature = "export-abi", test)), no_main)]
extern crate alloc;

// Modules and imports
mod erc721;

use crate::erc721::{Erc721, Erc721Error, Erc721Params};
use alloy_primitives::{Address, U256};
/// Import the Stylus SDK along with alloy primitive types for use in our program.
use stylus_sdk::prelude::*;

/// Immutable definitions
struct StylusTestNFTParams;
impl Erc721Params for StylusTestNFTParams {
    const NAME: &'static str = "StylusTestNFT";
    const SYMBOL: &'static str = "STNFT";

    fn token_uri(token_id: U256) -> String {
        format!("{}{}{}", "https://my-nft-metadata.com/", token_id, ".json")
    }
}

// Define the entrypoint as a Solidity storage object. The sol_storage! macro
// will generate Rust-equivalent structs with all fields mapped to Solidity-equivalent
// storage slots and types.
sol_storage! {
    #[entrypoint]
    struct StylusTestNFT {
        #[borrow] // Allows erc721 to access StylusTestNFT's storage and make calls
        Erc721<StylusTestNFTParams> erc721;
    }
}

#[public]
#[inherit(Erc721<StylusTestNFTParams>)]
impl StylusTestNFT {
    /// Mints an NFT
    pub fn mint(&mut self) -> Result<(), Erc721Error> {
        let minter = self.vm().msg_sender();
        self.erc721.mint(minter)?;
        Ok(())
    }

    /// Mints an NFT to another address
    pub fn mint_to(&mut self, to: Address) -> Result<(), Erc721Error> {
        self.erc721.mint(to)?;
        Ok(())
    }

    /// Burns an NFT
    pub fn burn(&mut self, token_id: U256) -> Result<(), Erc721Error> {
        // This function checks that msg_sender owns the specified token_id
        self.erc721.burn(self.vm().msg_sender(), token_id)?;
        Ok(())
    }

    /// Total supply
    pub fn total_supply(&mut self) -> Result<U256, Erc721Error> {
        Ok(self.erc721.total_supply.get())
    }
}

