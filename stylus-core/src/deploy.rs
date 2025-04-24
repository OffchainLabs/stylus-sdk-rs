// Copyright 2024-2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, B256, U256};

/// How to manage the storage cache, if enabled.
#[allow(unused)]
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum CachePolicy {
    #[default]
    DoNothing,
    Flush,
    Clear,
}

/// Provides the ability to deploy a smart contract with the given code and endowment value,
/// returning the address of the contract. Implementations must take care to handle
/// reentrancy and storage aliasing.
pub trait DeploymentAccess {
    #[cfg(feature = "reentrant")]
    /// Returns a contract deployer instance.
    /// Performs a raw deploy of another contract with the given `endowment` and init `code`.
    /// Returns the address of the newly deployed contract, or the error data in case of failure.
    ///
    /// # Safety
    ///
    /// Note that the EVM allows init code to make calls to other contracts, which provides a vector for
    /// reentrancy. This means that this method may enable storage aliasing if used in the middle of a storage
    /// reference's lifetime and if reentrancy is allowed.
    ///
    /// For extra flexibility, this method does not clear the global storage cache.
    unsafe fn deploy(
        &self,
        code: &[u8],
        endowment: U256,
        salt: Option<B256>,
        cache_policy: CachePolicy,
    ) -> Result<Address, Vec<u8>>;
    #[cfg(not(feature = "reentrant"))]
    /// Returns a contract deployer instance.
    /// Performs a raw deploy of another contract with the given `endowment` and init `code`.
    /// Returns the address of the newly deployed contract, or the error data in case of failure.
    ///
    /// # Safety
    ///
    /// Note that the EVM allows init code to make calls to other contracts, which provides a vector for
    /// reentrancy. This means that this method may enable storage aliasing if used in the middle of a storage
    /// reference's lifetime and if reentrancy is allowed.
    ///
    /// For extra flexibility, this method does not clear the global storage cache.
    unsafe fn deploy(
        &self,
        code: &[u8],
        endowment: U256,
        salt: Option<B256>,
    ) -> Result<Address, Vec<u8>>;
}
