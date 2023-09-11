// Copyright 2022-2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use crate::call::RawCall;
use alloc::vec::Vec;
use alloy_primitives::{Address, U256};

#[cfg(all(feature = "storage-cache", feature = "reentrant"))]
use crate::storage::TopLevelStorage;

#[cfg(all(feature = "storage-cache", feature = "reentrant"))]
use crate::storage::Storage;

/// Transfers an amount of ETH in wei to the given account.
/// Note that this method will call the other contract, which may in turn call others.
///
/// All gas is supplied, which the recipient may burn.
/// If this is not desired, the [`call`] method can be used directly.
#[cfg(all(feature = "storage-cache", feature = "reentrant"))]
pub fn transfer_eth(
    _storage: &mut impl TopLevelStorage,
    to: Address,
    amount: U256,
) -> Result<(), Vec<u8>> {
    Storage::clear(); // clear the storage to persist changes, invalidating the cache
    unsafe {
        RawCall::new_with_value(amount)
            .skip_return_data()
            .call(to, &[])?;
    }
    Ok(())
}

/// Transfers an amount of ETH in wei to the given account.
/// Note that this method will call the other contract, which may in turn call others.
///
/// All gas is supplied, which the recipient may burn.
/// If this is not desired, the [`call`] method can be used directly.
#[cfg(not(all(feature = "storage-cache", feature = "reentrant")))]
pub fn transfer_eth(to: Address, amount: U256) -> Result<(), Vec<u8>> {
    RawCall::new_with_value(amount)
        .skip_return_data()
        .call(to, &[])?;
    Ok(())
}
