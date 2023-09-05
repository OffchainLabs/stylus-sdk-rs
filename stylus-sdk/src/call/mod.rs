// Copyright 2022-2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use crate::storage::TopLevelStorage;
use alloy_primitives::{Address, U256};
use core::sync::atomic::{AtomicBool, Ordering};

pub use self::{context::Context, error::Error, raw::RawCall, traits::*};

pub(crate) use raw::CachePolicy;

#[cfg(feature = "storage-cache")]
use crate::storage::Storage;

mod context;
mod error;
mod raw;
mod traits;

/// Dangerous. Enables reentrancy.
///
/// # Safety
///
/// If a contract calls another that then calls the first, it is said to be reentrant.
/// By default, all Stylus programs revert when this happened.
/// This method overrides this behavior, allowing reentrant calls to proceed.
///
/// This is extremely dangerous, and should be done only after careful review --
/// ideally by 3rd party auditors. Numerous exploits and hacks have in Web3 are
/// attributable to developers misusing or not fully understanding reentrant patterns.
///
/// If enabled, the Stylus SDK will flush the storage cache in between reentrant calls,
/// persisting values to state that might be used by inner calls. Note that preventing storage
/// invalidation is only part of the battle in the fight against exploits.
pub unsafe fn opt_into_reentrancy() {
    ENABLE_REENTRANCY.store(true, Ordering::Relaxed)
}

/// Whether the program has opted into reentrancy.
pub fn reentrancy_enabled() -> bool {
    ENABLE_REENTRANCY.load(Ordering::Relaxed)
}

/// Whether the program has opted in to reentrancy.
static ENABLE_REENTRANCY: AtomicBool = AtomicBool::new(false);

/// Static calls the contract at the given address.
pub fn static_call(
    context: impl StaticCallContext,
    to: Address,
    data: &[u8],
) -> Result<Vec<u8>, Error> {
    #[cfg(feature = "storage-cache")]
    if reentrancy_enabled() {
        // flush storage to persist changes, but don't invalidate the cache
        Storage::flush();
    }
    unsafe {
        RawCall::new_static()
            .gas(context.gas())
            .call(to, data)
            .map_err(Error::Revert)
    }
}

/// Calls the contract at the given address.
pub fn call(context: impl MutatingCallContext, to: Address, data: &[u8]) -> Result<Vec<u8>, Error> {
    #[cfg(feature = "storage-cache")]
    if reentrancy_enabled() {
        // clear the storage to persist changes, invalidating the cache
        Storage::clear();
    }
    unsafe {
        RawCall::new_with_value(context.value())
            .gas(context.gas())
            .call(to, data)
            .map_err(Error::Revert)
    }
}

/// Transfers an amount of ETH in wei to the given account.
/// Note that this method will call the other contract, which may in turn call others.
///
/// All gas is supplied, which the recipient may burn.
/// If this is not desired, the [`call`] method can be used directly.
pub fn transfer_eth(
    _storage: &mut impl TopLevelStorage,
    to: Address,
    amount: U256,
) -> Result<(), Vec<u8>> {
    unsafe {
        RawCall::new_with_value(amount)
            .skip_return_data()
            .call(to, &[])?;
    }
    Ok(())
}
