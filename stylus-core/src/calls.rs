// Copyright 2025-2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy_primitives::{Address, U256};

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
    ) -> Result<Vec<u8>, Error>;
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
    ) -> Result<Vec<u8>, Error>;
    /// Calls the contract at the given address.
    fn call(
        &self,
        context: &dyn MutatingCallContext,
        to: Address,
        data: &[u8],
    ) -> Result<Vec<u8>, Error>;
}

/// Trait for transferring ETH.
pub trait ValueTransfer {
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

/// Trait for calling the `view` methods of other contracts.
/// Users should rarely implement this trait outside of proc macros.
pub trait StaticCallContext: CallContext {}

/// Trait for calling the mutable methods of other contracts.
/// Users should rarely implement this trait outside of proc macros.
///
/// # Safety
///
/// The type must contain a [`TopLevelStorage`][TLS] to prevent aliasing in cases of reentrancy.
///
/// [TLS]: stylus_core::context::TopLevelStorage
pub unsafe trait MutatingCallContext: CallContext {
    /// Amount of ETH in wei to give the other contract.
    fn value(&self) -> U256;
}

/// Trait for calling the `write` methods of other contracts.
/// Users should rarely implement this trait outside of proc macros.
///
/// Note: any implementations of this must return zero for [`MutatingCallContext::value`].
pub trait NonPayableCallContext: MutatingCallContext {}

use alloc::vec::Vec;
use alloy_sol_types::{Panic, PanicKind, SolError};

/// Represents error data when a call fails.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Revert data returned by the other contract.
    Revert(Vec<u8>),
    /// Failure to decode the other contract's return data.
    AbiDecodingFailed(alloy_sol_types::Error),
}

impl From<alloy_sol_types::Error> for Error {
    fn from(err: alloy_sol_types::Error) -> Self {
        Error::AbiDecodingFailed(err)
    }
}

/// Encode an error.
///
/// This is useful so that users can use `Error` as a variant in their error
/// types. It should not be necessary to implement this.
pub trait MethodError {
    /// Users should not have to call this.
    fn encode(self) -> Vec<u8>;
}

impl MethodError for Error {
    #[inline]
    fn encode(self) -> Vec<u8> {
        From::from(self)
    }
}

impl<T: SolError> MethodError for T {
    #[inline]
    fn encode(self) -> Vec<u8> {
        SolError::abi_encode(&self)
    }
}

impl From<Error> for Vec<u8> {
    #[allow(unused)]
    fn from(err: Error) -> Vec<u8> {
        match err {
            Error::Revert(data) => data,
            Error::AbiDecodingFailed(err) => Panic::from(PanicKind::Generic).abi_encode(),
        }
    }
}
