// Copyright 2022-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use crate::call::RawCall;
use crate::host::WasmVM;
use alloc::vec::Vec;
use alloy_primitives::{Address, U256};

#[cfg(feature = "reentrant")]
use stylus_core::context::TopLevelStorage;

#[cfg(feature = "reentrant")]
use crate::storage::Storage;

/// Transfers an amount of ETH in wei to the given account.
/// Note that this method will call the other contract, which may in turn call others.
///
/// All gas is supplied, which the recipient may burn.
/// If this is not desired, the [`call`](super::call) function may be used directly.
///
/// [`call`]: super::call
#[cfg(feature = "reentrant")]
#[deprecated(
    since = "0.8.0",
    note = "Use the .vm() method available on Stylus storage types along \
    with the stylus_core::ValueTransfer trait instead to transfer eth."
)]
pub fn transfer_eth(
    _storage: &mut impl TopLevelStorage,
    to: Address,
    amount: U256,
) -> Result<(), Vec<u8>> {
    Storage::clear(); // clear the storage to persist changes, invalidating the cache
    unsafe {
        RawCall::<WasmVM>::new_with_value(amount)
            .skip_return_data()
            .call(to, &[])?;
    }
    Ok(())
}

/// Transfers an amount of ETH in wei to the given account.
/// Note that this method will call the other contract, which may in turn call others.
///
/// All gas is supplied, which the recipient may burn.
/// If this is not desired, the [`call`](super::call) function may be used directly.
///
/// ```
/// # use stylus_sdk::call::{call, Call, transfer_eth};
/// # fn wrap() -> Result<(), Vec<u8>> {
/// #   let value = alloy_primitives::U256::ZERO;
/// #   let recipient = alloy_primitives::Address::ZERO;
/// transfer_eth(recipient, value)?;                 // these two are equivalent
/// call(Call::new().value(value), recipient, &[])?; // these two are equivalent
/// #     Ok(())
/// # }
/// ```
#[cfg(not(feature = "reentrant"))]
#[deprecated(
    since = "0.8.0",
    note = "Use the .vm() method available on Stylus storage types along \
    with the stylus_core::ValueTransfer trait instead to transfer eth."
)]
pub fn transfer_eth(to: Address, amount: U256) -> Result<(), Vec<u8>> {
    RawCall::<WasmVM>::new_with_value(amount)
        .skip_return_data()
        .call(to, &[])?;
    Ok(())
}
