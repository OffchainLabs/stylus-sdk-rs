// Copyright 2023, Offchain Labs, Inc.
// For license information, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use crate::hostio::{self, wrap_hostio};
use alloy_primitives::B256;

/// Emits an EVM log
pub fn log(topics: &[B256], data: &[u8]) -> Result<(), &'static str> {
    if topics.len() > 4 {
        return Err("too many topics");
    }
    let mut bytes: Vec<u8> = vec![];
    bytes.extend(topics.iter().flat_map(|x| x.0.iter()));
    bytes.extend(data);
    unsafe { hostio::emit_log(bytes.as_ptr(), bytes.len(), topics.len()) }
    Ok(())
}

wrap_hostio!(
    /// Gets the amount of gas remaining. See [`Ink and Gas`] for more information on Stylus's compute pricing.
    ///
    /// [`Ink and Gas`]: https://developer.arbitrum.io/TODO
    gas_left evm_gas_left u64
);

wrap_hostio!(
    /// Gets the amount of ink remaining. See [`Ink and Gas`] for more information on Stylus's compute pricing.
    ///
    /// [`Ink and Gas`]: https://developer.arbitrum.io/TODO
    ink_left evm_ink_left u64
);
