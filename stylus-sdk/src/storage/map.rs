// Copyright 2023, Offchain Labs, Inc.
// For license information, see https://github.com/OffchainLabs/nitro/blob/master/LICENSE

use super::{SizedStorageType, StorageGuardMut, StorageType};
use alloy_primitives::{FixedBytes, Signed, Uint, B256, U256};
use std::marker::PhantomData;

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
    pub fn open(&mut self, key: K) -> StorageGuardMut<V> {
        let slot = key.to_slot(self.slot.into());
        StorageGuardMut::new(V::new(slot, 0))
    }
}

impl<K: StorageKey, V: SizedStorageType> StorageMap<K, V> {
    pub fn insert(&mut self, key: K, value: V::Value) {
        let mut store = self.open(key);
        store.set_exact(value);
    }

    pub fn get(&self, key: K) -> V::Value {
        let slot = key.to_slot(self.slot.into());
        V::new(slot, 0).into()
    }
}

/// Trait that allows types to be the key of a [`StorageMap`].
pub trait StorageKey {
    fn to_slot(&self, root: B256) -> U256;
}

impl<const B: usize, const L: usize> StorageKey for Uint<B, L> {
    fn to_slot(&self, root: B256) -> U256 {
        let data = B256::from(U256::from(*self));
        data.concat_const::<32, 64>(root);
        data.into()
    }
}

impl<const B: usize, const L: usize> StorageKey for Signed<B, L> {
    fn to_slot(&self, root: B256) -> U256 {
        let data = B256::from(U256::from(self.into_raw()));
        data.concat_const::<32, 64>(root);
        data.into()
    }
}

impl<const N: usize> StorageKey for FixedBytes<N> {
    fn to_slot(&self, root: B256) -> U256 {
        let mut pad = [0; 32];
        pad[N..].copy_from_slice(&self.0);

        let data = B256::from(pad);
        data.concat_const::<32, 64>(root);
        data.into()
    }
}

impl StorageKey for bool {
    fn to_slot(&self, root: B256) -> U256 {
        let value = self.then_some(1_u8).unwrap_or_default();
        value.to_slot(root)
    }
}

macro_rules! impl_key {
    ($($ty:ident)+) => {
        $(impl StorageKey for $ty {
            fn to_slot(&self, root: B256) -> U256 {
                let data = B256::from(U256::from(*self));
                data.concat_const::<32, 64>(root.into());
                data.into()
            }
        })+
    };
}

impl_key!(u8 u16 u32 u64 usize);
