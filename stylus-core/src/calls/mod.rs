// Copyright 2025-2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy_primitives::{Address, U256};

pub mod context;
pub mod errors;

/// Trait for accessing a safe API for calling other contracts.
/// Its implementation should have reentrancy awareness and protections depending
/// on the SDK configuration.
pub trait CallAccess {
    /// Static calls the contract at the given address.
    fn static_call(
        &self,
        context: &dyn StaticCallContext,
        to: Address,
        data: &[u8],
    ) -> Result<Vec<u8>, errors::Error>;
    /// Delegate calls the contract at the given address.
    ///
    /// # Safety
    ///
    /// A delegate call must trust the other contract to uphold safety requirements.
    /// Though this function clears any cached values, the other contract may arbitrarily change storage,
    /// spend ether, and do other things one should never blindly allow other contracts to do.
    unsafe fn delegate_call(
        &self,
        context: &dyn MutatingCallContext,
        to: Address,
        data: &[u8],
    ) -> Result<Vec<u8>, errors::Error>;
    /// Calls the contract at the given address.
    fn call(
        &self,
        context: &dyn MutatingCallContext,
        to: Address,
        data: &[u8],
    ) -> Result<Vec<u8>, errors::Error>;
}

/// Trait for transferring ETH.
pub trait ValueTransfer {
    #[cfg(feature = "reentrant")]
    /// Transfers an amount of ETH in wei to the given account.
    /// Note that this method will call the other contract, which may in turn call others.
    ///
    /// All gas is supplied, which the recipient may burn.
    /// If this is not desired, the [`call`] method on the CallAccess trait may be used directly.
    fn transfer_eth(
        &self,
        storage: &mut dyn crate::storage::TopLevelStorage,
        to: Address,
        amount: U256,
    ) -> Result<(), Vec<u8>>;
    #[cfg(not(feature = "reentrant"))]
    /// Transfers an amount of ETH in wei to the given account.
    /// Note that this method will call the other contract, which may in turn call others.
    ///
    /// All gas is supplied, which the recipient may burn.
    /// If this is not desired, the [`call`] method on the CallAccess trait may be used directly.
    fn transfer_eth(&self, to: Address, amount: U256) -> Result<(), Vec<u8>>;
}

/// Trait for calling other contracts.
/// Users should rarely implement this trait outside of proc macros.
pub trait CallContext {
    /// Amount of gas to supply the call.
    /// Note: values are clipped to the amount of gas remaining.
    fn gas(&self) -> u64;
}

/// Trait for calling the `view` or `pure` methods of other contracts.
/// Users should rarely implement this trait outside of proc macros.
pub trait StaticCallContext: CallContext {}

/// Trait for calling the mutable methods of other contracts.
/// Users should rarely implement this trait outside of proc macros.
///
/// # Safety
///
/// The type must contain a [`TopLevelStorage`][TLS] to prevent aliasing in cases of reentrancy.
///
/// [TLS]: stylus_core::storage::TopLevelStorage
pub unsafe trait MutatingCallContext: CallContext {
    /// Amount of ETH in wei to give the other contract.
    fn value(&self) -> U256;
}

/// Trait for calling the `write` methods of other contracts.
/// Users should rarely implement this trait outside of proc macros.
///
/// Note: any implementations of this must return zero for [`MutatingCallContext::value`].
pub trait NonPayableCallContext: MutatingCallContext {}
