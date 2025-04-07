// Copyright 2022-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Call other contracts.
//!
//! There are two primary ways to make calls to other contracts via the Stylus SDK.
//! - [`Call`] with [`sol_interface!`][sol_interface] for richly-typed calls.
//! - [`RawCall`] for `unsafe`, bytes-in bytes-out calls.
//!
//! Additional helpers exist for specific use-cases like [`transfer_eth`].
//!
//! [sol_interface]: crate::prelude::sol_interface

use alloc::vec::Vec;
use alloy_primitives::Address;

pub use self::{
    context::Call, error::Error, error::MethodError, raw::RawCall, traits::*,
    transfer::transfer_eth,
};
use crate::storage::Storage;

pub(crate) use raw::CachePolicy;

mod context;
mod error;
mod raw;
mod traits;
mod transfer;

/// Static calls the contract at the given address.
pub fn static_call(
    context: impl StaticCallContext,
    to: Address,
    data: &[u8],
) -> Result<Vec<u8>, Error> {
    Storage::flush(); // flush storage to persist changes, but don't invalidate the cache

    unsafe {
        RawCall::new_static()
            .gas(context.gas())
            .call(to, data)
            .map_err(Error::Revert)
    }
}

/// Delegate calls the contract at the given address.
///
/// # Safety
///
/// A delegate call must trust the other contract to uphold safety requirements.
/// Though this function clears any cached values, the other contract may arbitrarily change storage,
/// spend ether, and do other things one should never blindly allow other contracts to do.
pub unsafe fn delegate_call(
    context: impl MutatingCallContext,
    to: Address,
    data: &[u8],
) -> Result<Vec<u8>, Error> {
    Storage::clear(); // clear the storage to persist changes, invalidating the cache

    unsafe {
        RawCall::new_delegate()
            .gas(context.gas())
            .call(to, data)
            .map_err(Error::Revert)
    }
}

/// Calls the contract at the given address.
pub fn call(context: impl MutatingCallContext, to: Address, data: &[u8]) -> Result<Vec<u8>, Error> {
    Storage::clear(); // clear the storage to persist changes, invalidating the cache

    unsafe {
        RawCall::new_with_value(context.value())
            .gas(context.gas())
            .call(to, data)
            .map_err(Error::Revert)
    }
}
