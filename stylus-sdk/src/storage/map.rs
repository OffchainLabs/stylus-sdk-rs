// Copyright 2023, Offchain Labs, Inc.
// For license information, see https://github.com/OffchainLabs/nitro/blob/master/LICENSE

use super::{SizedStorageType, StorageType};
use alloy_primitives::U256;
use std::marker::PhantomData;

pub trait StorageKey {
    fn hash(&self, root: U256) -> U256 {
        root
    }
}

/// Accessor for a storage-backed map
pub struct StorageMap<K: StorageKey, V: StorageType> {
    slot: U256,
    marker: PhantomData<(K, V)>,
}

impl<K: StorageKey, V: StorageType> StorageType for StorageMap<K, V> {
    fn new(slot: U256, offset: u8) -> Self {
        debug_assert!(offset == 0);
        Self {
            slot,
            marker: PhantomData,
        }
    }
}

impl<K: StorageKey, V: StorageType> StorageMap<K, V> {
    pub fn open(&mut self, key: K) -> V {
        let slot = key.hash(self.slot);
        V::new(slot, 0)
    }
}

impl<K: StorageKey, V: SizedStorageType> StorageMap<K, V> {
    pub fn insert(&mut self, key: K, value: V::Value) {
        let mut store = self.open(key);
        store.set_exact(value);
    }
}
