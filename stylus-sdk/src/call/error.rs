// Copyright 2022-2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use alloy_sol_types::{Panic, PanicKind, SolError};

/// Represents error data when a call fails.
#[derive(Debug, PartialEq)]
pub enum Error {
    Revert(Vec<u8>),
    AbiDecodingFailed(alloy_sol_types::Error),
}

impl From<alloy_sol_types::Error> for Error {
    fn from(err: alloy_sol_types::Error) -> Self {
        Error::AbiDecodingFailed(err)
    }
}

#[allow(unused)]
impl From<Error> for Vec<u8> {
    fn from(err: Error) -> Vec<u8> {
        match err {
            Error::Revert(data) => data,
            Error::AbiDecodingFailed(err) => {
                console!("failed to decode return data from external call: {err}");
                Panic::from(PanicKind::Generic).encode()
            }
        }
    }
}
