// Copyright 2023, Offchain Labs, Inc.
// For license information, see https://github.com/OffchainLabs/nitro/blob/master/LICENSE

use crate::crypto;

use super::{SizedStorageType, StorageGuard, StorageGuardMut, StorageType};
use alloy_primitives::{Address, FixedBytes, Signed, Uint, B256, U160, U256};
use std::marker::PhantomData;

/// Accessor for a storage-backed map
pub struct StorageMap<K: StorageKey, V: StorageType> {
    slot: U256,
    marker: PhantomData<(K, V)>,
}

impl<K: StorageKey, V: StorageType> StorageType for StorageMap<K, V> {
    type Wraps<'a> = StorageGuard<'a, StorageMap<K, V>> where Self: 'a;
    type WrapsMut<'a> = StorageGuardMut<'a, StorageMap<K, V>> where Self: 'a;

    unsafe fn new(slot: U256, offset: u8) -> Self {
        debug_assert!(offset == 0);
        Self {
            slot,
            marker: PhantomData,
        }
    }

    fn load<'s>(self) -> Self::Wraps<'s> {
        StorageGuard::new(self)
    }

    fn load_mut<'s>(self) -> Self::WrapsMut<'s> {
        StorageGuardMut::new(self)
    }
}

impl<K: StorageKey, V: StorageType> StorageMap<K, V> {
    const CHILD_OFFSET: u8 = 32 - V::SLOT_BYTES as u8;

    pub fn getter(&self, key: K) -> StorageGuard<V> {
        let slot = key.to_slot(self.slot.into());
        unsafe { StorageGuard::new(V::new(slot, Self::CHILD_OFFSET)) }
    }

    pub fn setter(&mut self, key: K) -> StorageGuardMut<V> {
        let slot = key.to_slot(self.slot.into());
        unsafe { StorageGuardMut::new(V::new(slot, Self::CHILD_OFFSET)) }
    }
}

impl<'a, K: StorageKey, V: SizedStorageType<'a>> StorageMap<K, V> {
    pub fn insert(&mut self, key: K, value: V::Wraps<'a>) {
        let mut store = self.setter(key);
        store.set_exact(value);
    }

    pub fn get(&self, key: K) -> V::Wraps<'a> {
        let store = self.getter(key);
        unsafe { store.into_raw().load() }
    }
}

/// Trait that allows types to be the key of a [`StorageMap`].
/// Note: the assignment of slots must be injective.
pub trait StorageKey {
    fn to_slot(&self, root: B256) -> U256;
}

impl<const B: usize, const L: usize> StorageKey for Uint<B, L> {
    fn to_slot(&self, root: B256) -> U256 {
        let data = B256::from(U256::from(*self));
        let data = data.concat_const::<32, 64>(root);
        crypto::keccak(data).into()
    }
}

impl<const B: usize, const L: usize> StorageKey for Signed<B, L> {
    fn to_slot(&self, root: B256) -> U256 {
        let data = B256::from(U256::from(self.into_raw()));
        let data = data.concat_const::<32, 64>(root);
        crypto::keccak(data).into()
    }
}

impl<const N: usize> StorageKey for FixedBytes<N> {
    fn to_slot(&self, root: B256) -> U256 {
        let mut pad = [0; 32];
        pad[..N].copy_from_slice(&self.0);

        let data = B256::from(pad);
        let data = data.concat_const::<32, 64>(root);
        crypto::keccak(data).into()
    }
}

impl StorageKey for &[u8] {
    fn to_slot(&self, root: B256) -> U256 {
        let mut vec = self.to_vec();
        vec.extend(root);
        crypto::keccak(vec).into()
    }
}

impl StorageKey for Vec<u8> {
    fn to_slot(&self, root: B256) -> U256 {
        let bytes: &[u8] = self.as_ref();
        bytes.to_slot(root)
    }
}

impl StorageKey for &str {
    fn to_slot(&self, root: B256) -> U256 {
        self.as_bytes().to_slot(root)
    }
}

impl StorageKey for String {
    fn to_slot(&self, root: B256) -> U256 {
        self.as_bytes().to_slot(root)
    }
}

impl StorageKey for Address {
    fn to_slot(&self, root: B256) -> U256 {
        let int: U160 = self.0.try_into().unwrap();
        int.to_slot(root)
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
                let data = data.concat_const::<32, 64>(root.into());
                crypto::keccak(data).into()
            }
        })+
    };
}

impl_key!(u8 u16 u32 u64 usize);
