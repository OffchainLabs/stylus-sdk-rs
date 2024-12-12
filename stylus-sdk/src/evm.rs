// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Affordances for the Ethereum Virtual Machine.
//!
//! See also [`block`](crate::block), [`contract`](crate::contract), [`crypto`](crate::crypto),
//! [`msg`](crate::msg), and [`tx`](crate::msg).
//!
//! ```no_run
//! use stylus_sdk::evm;
//!
//! let gas = evm::gas_left();
//! ```

use crate::hostio::{self, wrap_hostio};
use alloc::{vec, vec::Vec};
use alloy_primitives::B256;
use alloy_sol_types::{abi::token::WordToken, SolEvent, TopicList};

/// Emits an evm log from combined topics and data.
fn emit_log(bytes: &[u8], num_topics: usize) {
    unsafe { hostio::emit_log(bytes.as_ptr(), bytes.len(), num_topics) }
}

/// Emits an EVM log from its raw topics and data.
/// Most users should prefer the alloy-typed [`log`].
pub fn raw_log(topics: &[B256], data: &[u8]) -> Result<(), &'static str> {
    if topics.len() > 4 {
        return Err("too many topics");
    }
    let mut bytes: Vec<u8> = vec![];
    bytes.extend(topics.iter().flat_map(|x| x.0.iter()));
    bytes.extend(data);
    emit_log(&bytes, topics.len());
    Ok(())
}

/// Emits a typed alloy log.
pub fn log<T: SolEvent>(event: T) {
    // According to the alloy docs, encode_topics_raw fails only if the array is too small

    let mut topics = [WordToken::default(); 4];
    event.encode_topics_raw(&mut topics).unwrap();

    let count = T::TopicList::COUNT;
    let mut bytes = Vec::with_capacity(32 * count);
    for topic in &topics[..count] {
        bytes.extend_from_slice(topic.as_slice());
    }
    event.encode_data_to(&mut bytes);
    emit_log(&bytes, count);
}

/// This function exists to force the compiler to import this symbol.
/// Calling it will unproductively consume gas.
pub fn pay_for_memory_grow(pages: u16) {
    unsafe { hostio::pay_for_memory_grow(pages) }
}

wrap_hostio!(
    /// Gets the amount of gas remaining. See [`Ink and Gas`] for more information on Stylus's compute pricing.
    ///
    /// [`Ink and Gas`]: https://docs.arbitrum.io/stylus/concepts/gas-metering
    gas_left evm_gas_left u64
);

wrap_hostio!(
    /// Gets the amount of ink remaining. See [`Ink and Gas`] for more information on Stylus's compute pricing.
    ///
    /// [`Ink and Gas`]: https://docs.arbitrum.io/stylus/concepts/gas-metering
    ink_left evm_ink_left u64
);
