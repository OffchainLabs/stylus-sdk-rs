// Copyright 2025-2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md
//
extern crate alloc;

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
