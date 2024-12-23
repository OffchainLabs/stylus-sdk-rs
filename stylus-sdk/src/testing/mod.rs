// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Defines a testing framework for Stylus contracts. Allows for easy unit testing
//! of programs by using a MockHost which implements the [`Host`] trait and provides
//! Foundry-style cheatcodes for programs.

use alloy_primitives::{Address, FixedBytes, U256};

/// The host trait defines methods a contract can use to interact
/// with a host environment, such as the EVM.
use crate::host::Host;

/// Extends the host trait to implement Foundry-style cheatcodes.
pub trait CheatcodeProvider: Host {
    /// Set block.timestamp
    fn warp(&mut self, t: U256);

    /// Set block.number
    fn roll(&mut self, block_num: U256);

    /// Set block.basefee
    fn fee(&mut self, fee: U256);

    /// Set block.difficulty
    /// Does not work from the Paris hard fork and onwards, and will revert instead.
    fn difficulty(&mut self, diff: U256);

    /// Set block.prevrandao
    /// Does not work before the Paris hard fork, and will revert instead.
    fn prevrandao(&mut self, randao: FixedBytes<32>);

    /// Set block.chainid
    fn chain_id(&mut self, id: U256);

    /// Loads a storage slot from an address
    fn load(&self, account: Address, slot: FixedBytes<32>) -> FixedBytes<32>;

    /// Stores a value to an address' storage slot
    fn store(&mut self, account: Address, slot: FixedBytes<32>, value: FixedBytes<32>);

    /// Sets the *next* call's msg.sender to be the input address
    fn prank(&mut self, sender: Address);
}

#[derive(Default)]
pub struct MockHost;

//impl Host for MockHost {
//    fn msg_sender() -> Address {
//        Address::ZERO
//    }
//}
//
//impl CheatcodeProvider for MockHost {
//    fn warp(&mut self, t: U256) {}
//    fn roll(&mut self, block_num: U256) {}
//    fn fee(&mut self, fee: U256) {}
//    fn difficulty(&mut self, diff: U256) {}
//    fn prevrandao(&mut self, randao: FixedBytes<32>) {}
//    fn chain_id(&mut self, id: U256) {}
//    fn load(&self, account: Address, slot: FixedBytes<32>) -> FixedBytes<32> {
//        slot
//    }
//    fn store(&mut self, account: Address, slot: FixedBytes<32>, value: FixedBytes<32>) {}
//    fn prank(&mut self, sender: Address) {}
//}
