// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy_primitives::{Address, Bytes, FixedBytes, U256};
use alloy_sol_types::sol;
use stylus_sdk::prelude::*;

// Declare Solidity error types
sol! {
    // Token id has not been minted, or it has been burned
    error InvalidTokenId(uint256 token_id);
    // The specified address is not the owner of the specified token id
    error NotOwner(address from, uint256 token_id, address real_owner);
    // The specified address does not have allowance to spend the specified token id
    error NotApproved(address owner, address spender, uint256 token_id);
    // Attempt to transfer token id to the Zero address
    error TransferToZero(uint256 token_id);
    // The receiver address refused to receive the specified token id
    error ReceiverRefused(address receiver, uint256 token_id, bytes4 returned);
}

/// Represents the ways methods may fail.
#[derive(SolidityError)]
pub enum Erc721Error {
    InvalidTokenId(InvalidTokenId),
    NotOwner(NotOwner),
    NotApproved(NotApproved),
    TransferToZero(TransferToZero),
    ReceiverRefused(ReceiverRefused),
}

// Trait that contains the Erc721 methods.
#[public]
pub trait IErc721 {
    /// Immutable NFT name.
    fn name(&self) -> Result<String, Erc721Error>;

    /// Immutable NFT symbol.
    fn symbol(&self) -> Result<String, Erc721Error>;

    /// The NFT's Uniform Resource Identifier.
    fn token_uri(&self, token_id: U256) -> Result<String, Erc721Error>;

    /// Gets the number of NFTs owned by an account.
    fn balance_of(&self, owner: Address) -> Result<U256, Erc721Error>;

    /// Gets the owner of the NFT, if it exists.
    fn owner_of(&self, token_id: U256) -> Result<Address, Erc721Error>;

    /// Transfers an NFT, but only after checking the `to` address can receive the NFT.
    /// It includes additional data for the receiver.
    fn safe_transfer_from_with_data(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Erc721Error>;

    /// Equivalent to [`safe_transfer_from_with_data`], but without the additional data.
    ///
    /// Note: because Rust doesn't allow multiple methods with the same name,
    /// we use the `#[selector]` macro attribute to simulate solidity overloading.
    fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Erc721Error>;

    /// Transfers the NFT.
    fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Erc721Error>;

    /// Grants an account the ability to manage the sender's NFT.
    fn approve(&mut self, approved: Address, token_id: U256) -> Result<(), Erc721Error>;

    /// Grants an account the ability to manage all of the sender's NFTs.
    fn set_approval_for_all(
        &mut self,
        operator: Address,
        approved: bool,
    ) -> Result<(), Erc721Error>;

    /// Gets the account managing an NFT, or zero if unmanaged.
    fn get_approved(&mut self, token_id: U256) -> Result<Address, Erc721Error>;

    /// Determines if an account has been authorized to managing all of a user's NFTs.
    fn is_approved_for_all(
        &mut self,
        owner: Address,
        operator: Address,
    ) -> Result<bool, Erc721Error>;

    /// Whether the NFT supports a given standard.
    fn supports_interface(&self, interface: FixedBytes<4>) -> Result<bool, Erc721Error>;
}
