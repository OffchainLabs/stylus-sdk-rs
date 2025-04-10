// Copyright 2024-2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md
#![no_std]

//! Defines host environment methods Stylus SDK contracts have access to.

extern crate alloc;

pub mod calls;
pub mod deploy;
pub mod host;
pub mod storage;

use alloy_sol_types::{abi::token::WordToken, SolEvent, TopicList};

use alloc::vec::Vec;

pub use host::*;
pub use storage::TopLevelStorage;

/// Emits a typed, Alloy log.
pub fn log<T: SolEvent>(vm: &dyn Host, event: T) {
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
