// Copyright 2024-2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Defines host environment methods Stylus SDK contracts have access to.
pub mod calls;
pub mod deploy;
pub mod host;
pub mod storage;

use alloy_sol_types::{abi::token::WordToken, SolEvent, TopicList};
pub use host::*;

/// Emits a typed, Alloy log.
pub fn log<T: SolEvent>(vm: &impl Host, event: T) {
    // According to the alloy docs, encode_topics_raw fails only if the array is too small
    let mut topics = [WordToken::default(); 4];
    event.encode_topics_raw(&mut topics).unwrap();

    let count = T::TopicList::COUNT;
    let mut bytes = Vec::with_capacity(32 * count);
    for topic in &topics[..count] {
        bytes.extend_from_slice(topic.as_slice());
    }
    event.encode_data_to(&mut bytes);
    vm.emit_log(&bytes, count);
}
