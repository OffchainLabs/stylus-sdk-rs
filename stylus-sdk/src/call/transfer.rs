// Copyright 2022-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloc::vec::Vec;
use alloy_primitives::{Address, U256};
use stylus_core::Host;

use super::RawCall;

/// Transfers an amount of ETH in wei to the given account.
/// Note that this method will call the other contract, which may in turn call others.
///
/// All gas is supplied, which the recipient may burn.
/// If this is not desired, the [`call`](super::call) function may be used directly.
///
/// ```
/// # use stylus_sdk::prelude::*;
/// # use stylus_sdk::stylus_core::host::Host;
/// # use stylus_sdk::call::transfer::transfer_eth;
/// # fn wrap(host: &dyn Host) -> Result<(), Vec<u8>> {
/// #   let value = alloy_primitives::U256::ZERO;
/// #   let recipient = alloy_primitives::Address::ZERO;
/// transfer_eth(host, recipient, value)?;                 // these two are equivalent
/// call(host, Call::new().value(value), recipient, &[])?; // these two are equivalent
/// #     Ok(())
/// # }
/// ```
pub fn transfer_eth<H: Host>(host: &H, to: Address, amount: U256) -> Result<(), Vec<u8>> {
    host.flush_cache(true); // clear the storage to persist changes, invalidating the cache
    unsafe {
        RawCall::new_with_value(host, amount)
            .skip_return_data()
            .call(to, &[])?;
    }
    Ok(())
}
