use alloy_primitives::{Address, U256};

pub trait CallAccess {
    fn static_call(
        &self,
        context: impl StaticCallContext,
        to: Address,
        data: &[u8],
    ) -> Result<Vec<u8>, Error>;
    fn delegate_call(
        &self,
        context: impl MutatingCallContext,
        to: Address,
        data: &[u8],
    ) -> Result<Vec<u8>, Error>;
    fn call(
        &self,
        context: impl MutatingCallContext,
        to: Address,
        data: &[u8],
    ) -> Result<Vec<u8>, Error>;
}

pub trait ValueTransfer {
    fn transfer_eth(to: Address, amount: U256) -> Result<(), Vec<u8>>;
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
/// [TLS]: crate::storage::TopLevelStorage
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
