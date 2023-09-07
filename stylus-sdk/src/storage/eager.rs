// Copyright 2022-2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use super::{load_bytes32, store_bytes32, traits::GlobalStorage};
use alloy_primitives::{B256, U256};

/// Global accessor to persistent storage that doesn't use caching.
///
/// To instead use storage-caching optimizations, recompile with the
/// `storage-cache` feature flag, which will provide the [`StorageCache`] type.
///
/// Note that individual primitive types may still include efficient caching.
///
/// [`StorageCache`]: super::StorageCache
pub struct EagerStorage;

impl GlobalStorage for EagerStorage {
    fn get_word(key: U256) -> B256 {
        unsafe { load_bytes32(key) }
    }

    unsafe fn set_word(key: U256, value: B256) {
        store_bytes32(key, value);
    }
}
