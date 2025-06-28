// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

// Only run this as a WASM if the export-abi feature is not set.
#![cfg_attr(not(any(feature = "export-abi", test)), no_main)]
extern crate alloc;

// Modules and imports
pub mod erc721;
pub mod ierc721;

use crate::erc721::{Erc721, Erc721Params};
use crate::ierc721::{Erc721Error, IErc721};
use alloy_primitives::{Address, Bytes, FixedBytes, U256};
/// Import the Stylus SDK along with alloy primitive types for use in our program.
use stylus_sdk::prelude::*;

/// Immutable definitions
pub struct StylusTestNFTParams;
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
#[implements(IErc721)]
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

#[public]
impl IErc721 for StylusTestNFT {
    fn name(&self) -> Result<String, Erc721Error> {
        Erc721::<StylusTestNFTParams>::name()
    }

    fn symbol(&self) -> Result<String, Erc721Error> {
        Erc721::<StylusTestNFTParams>::symbol()
    }

    #[selector(name = "tokenURI")]
    fn token_uri(&self, token_id: U256) -> Result<String, Erc721Error> {
        self.erc721.token_uri(token_id)
    }

    fn balance_of(&self, owner: Address) -> Result<U256, Erc721Error> {
        self.erc721.balance_of(owner)
    }

    fn owner_of(&self, token_id: U256) -> Result<Address, Erc721Error> {
        self.erc721.owner_of(token_id)
    }

    #[selector(name = "safeTransferFrom")]
    fn safe_transfer_from_with_data(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Erc721Error> {
        Erc721::<StylusTestNFTParams>::safe_transfer_from_with_data(self, from, to, token_id, data)
    }

    #[selector(name = "safeTransferFrom")]
    fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Erc721Error> {
        Erc721::<StylusTestNFTParams>::safe_transfer_from(self, from, to, token_id)
    }

    fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Erc721Error> {
        self.erc721.transfer_from(from, to, token_id)
    }

    fn approve(&mut self, approved: Address, token_id: U256) -> Result<(), Erc721Error> {
        self.erc721.approve(approved, token_id)
    }

    fn set_approval_for_all(
        &mut self,
        operator: Address,
        approved: bool,
    ) -> Result<(), Erc721Error> {
        self.erc721.set_approval_for_all(operator, approved)
    }

    fn get_approved(&mut self, token_id: U256) -> Result<Address, Erc721Error> {
        self.erc721.get_approved(token_id)
    }

    fn is_approved_for_all(
        &mut self,
        owner: Address,
        operator: Address,
    ) -> Result<bool, Erc721Error> {
        self.erc721.is_approved_for_all(owner, operator)
    }

    fn supports_interface(&self, interface: FixedBytes<4>) -> Result<bool, Erc721Error> {
        Erc721::<StylusTestNFTParams>::supports_interface(interface)
    }
}
