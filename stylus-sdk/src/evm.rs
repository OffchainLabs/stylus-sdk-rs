// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Affordances for the Ethereum Virtual Machine.

use alloc::vec::Vec;
use alloy_sol_types::{abi::token::WordToken, SolEvent, TopicList};
use stylus_core::Host;

/// Emits a typed alloy log.
pub fn log<T: SolEvent, H: Host>(host: &H, event: T) {
    // According to the alloy docs, encode_topics_raw fails only if the array is too small

    let mut topics = [WordToken::default(); 4];
    event.encode_topics_raw(&mut topics).unwrap();

    let count = T::TopicList::COUNT;
    let mut bytes = Vec::with_capacity(32 * count);
    for topic in &topics[..count] {
        bytes.extend_from_slice(topic.as_slice());
    }
    event.encode_data_to(&mut bytes);
    host.emit_log(&bytes, count);
}
