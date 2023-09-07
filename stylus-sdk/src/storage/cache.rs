// Copyright 2022-2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use super::{load_bytes32, store_bytes32, traits::GlobalStorage};
use alloy_primitives::{B256, U256};
use core::cell::UnsafeCell;
use fnv::FnvHashMap as HashMap;
use lazy_static::lazy_static;

/// Global cache managing persistent storage operations.
///
/// This is intended for most use cases. However, one may opt-out
/// of this behavior by turning off default features and not enabling
/// the `storage-cache` feature. Doing so will provide the [`EagerStorage`]
/// type for managing state in the absence of caching.
///
/// [`EagerStorage`]: super::EagerStorage
pub struct StorageCache(HashMap<U256, StorageWord>);

/// Represents the EVM word at a given key.
pub struct StorageWord {
    /// The current value of the slot.
    value: B256,
    /// The value in the EVM state trie, if known.
    known: Option<B256>,
}

impl StorageWord {
    /// Creates a new slot from a known value in the EVM state trie.
    fn new_known(known: B256) -> Self {
        Self {
            value: known,
            known: Some(known),
        }
    }

    /// Creates a new slot without knowing the underlying value in the EVM state trie.
    fn new_unknown(value: B256) -> Self {
        Self { value, known: None }
    }

    /// Whether a slot should be written to disk.
    fn dirty(&self) -> bool {
        Some(self.value) != self.known
    }
}

/// Forces a type to implement [`Sync`].
struct ForceSync<T>(T);

unsafe impl<T> Sync for ForceSync<T> {}

lazy_static! {
    /// Global cache managing persistent storage operations.
    static ref CACHE: ForceSync<UnsafeCell<StorageCache>> = ForceSync(UnsafeCell::new(StorageCache(HashMap::default())));
}

/// Mutably accesses the global cache's hashmap
macro_rules! cache {
    () => {
        unsafe { &mut (*CACHE.0.get()).0 }
    };
}

impl GlobalStorage for StorageCache {
    fn get_word(key: U256) -> B256 {
        cache!()
            .entry(key)
            .or_insert_with(|| unsafe { StorageWord::new_known(load_bytes32(key)) })
            .value
    }

    unsafe fn set_word(key: U256, value: B256) {
        cache!().insert(key, StorageWord::new_unknown(value));
    }
}

impl StorageCache {
    /// Write all cached values to persistent storage.
    /// Note: this operation retains [`SLOAD`] information for optimization purposes.
    /// If reentrancy is possible, use [`StorageCache::clear`].
    ///
    /// [`SLOAD`]: https://www.evm.codes/#54
    pub fn flush() {
        for (key, entry) in cache!() {
            if entry.dirty() {
                unsafe { store_bytes32(*key, entry.value) };
            }
        }
    }

    /// Flush and clear the storage cache.
    pub fn clear() {
        StorageCache::flush();
        cache!().clear();
    }
}
