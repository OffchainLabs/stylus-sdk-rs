// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use crate::crypto;
use crate::host::VM;

use super::{Erase, SimpleStorageType, StorageGuard, StorageGuardMut, StorageType};
use alloc::{string::String, vec::Vec};
use alloy_primitives::{Address, FixedBytes, Signed, Uint, B256, U160, U256};
use core::marker::PhantomData;
use stylus_core::HostAccess;

/// Accessor for a storage-backed map.
pub struct StorageMap<K: StorageKey, V: StorageType> {
    slot: U256,
    marker: PhantomData<(K, V)>,
    __stylus_host: VM,
}

impl<K, V> StorageType for StorageMap<K, V>
where
    K: StorageKey,
    V: StorageType,
{
    type Wraps<'a>
        = StorageGuard<'a, StorageMap<K, V>>
    where
        Self: 'a;
    type WrapsMut<'a>
        = StorageGuardMut<'a, StorageMap<K, V>>
    where
        Self: 'a;

    unsafe fn new(slot: U256, offset: u8, host: VM) -> Self {
        debug_assert!(offset == 0);
        Self {
            slot,
            marker: PhantomData,
            __stylus_host: host,
        }
    }

    fn load<'s>(self) -> Self::Wraps<'s> {
        StorageGuard::new(self)
    }

    fn load_mut<'s>(self) -> Self::WrapsMut<'s> {
        StorageGuardMut::new(self)
    }
}

impl<K, V> HostAccess for StorageMap<K, V>
where
    K: StorageKey,
    V: StorageType,
{
    type Host = VM;

    #[inline]
    fn vm(&self) -> &Self::Host {
        &self.__stylus_host
    }
}

#[cfg(feature = "stylus-test")]
impl<K, V, T> From<&T> for StorageMap<K, V>
where
    K: StorageKey,
    V: StorageType,
    T: stylus_core::Host + Clone + 'static,
{
    fn from(host: &T) -> Self {
        unsafe {
            Self::new(
                U256::ZERO,
                0,
                crate::host::VM {
                    host: alloc::boxed::Box::new(host.clone()),
                },
            )
        }
    }
}

impl<K, V> StorageMap<K, V>
where
    K: StorageKey,
    V: StorageType,
{
    /// Where in a word to access the wrapped value.
    const CHILD_OFFSET: u8 = 32 - V::SLOT_BYTES as u8;

    /// Gets an accessor to the element at the given key, or the zero-value if none is there.
    /// Note: the accessor is protected by a [`StorageGuard`], which restricts its lifetime
    /// to that of `&self`.
    pub fn getter(&self, key: K) -> StorageGuard<'_, V> {
        let slot = key.to_slot(self.slot.into());
        unsafe { StorageGuard::new(V::new(slot, Self::CHILD_OFFSET, self.__stylus_host.clone())) }
    }

    /// Gets a mutable accessor to the element at the given key, or the zero-value if none is there.
    /// Note: the accessor is protected by a [`StorageGuardMut`], which restricts its lifetime
    /// to that of `&mut self`.
    pub fn setter(&mut self, key: K) -> StorageGuardMut<'_, V> {
        let slot = key.to_slot(self.slot.into());
        unsafe {
            StorageGuardMut::new(V::new(slot, Self::CHILD_OFFSET, self.__stylus_host.clone()))
        }
    }

    /// Gets the element at the given key, or the zero value if none is there.
    pub fn get(&self, key: K) -> V::Wraps<'_> {
        let store = self.getter(key);
        unsafe { store.into_raw().load() }
    }
}

impl<'a, K, V> StorageMap<K, V>
where
    K: StorageKey,
    V: SimpleStorageType<'a>,
{
    /// Sets the element at a given key, overwriting what may have been there.
    pub fn insert(&mut self, key: K, value: V::Wraps<'a>) {
        let mut store = self.setter(key);
        store.set_by_wrapped(value);
    }

    /// Replace the element at the given key.
    /// Returns the old element, or the zero-value if none was there.
    pub fn replace(&mut self, key: K, value: V::Wraps<'a>) -> V::Wraps<'a> {
        let slot = key.to_slot(self.slot.into());
        // intentionally alias so that we can erase after load
        unsafe {
            let store = V::new(slot, Self::CHILD_OFFSET, self.__stylus_host.clone());
            let mut alias = V::new(slot, Self::CHILD_OFFSET, self.__stylus_host.clone());
            let prior = store.load();
            alias.set_by_wrapped(value);
            prior
        }
    }

    /// Remove the element at the given key.
    /// Returns the element, or the zero-value if none was there.
    pub fn take(&mut self, key: K) -> V::Wraps<'a> {
        let slot = key.to_slot(self.slot.into());
        // intentionally alias so that we can erase after load
        unsafe {
            let store = V::new(slot, Self::CHILD_OFFSET, self.__stylus_host.clone());
            let mut alias = V::new(slot, Self::CHILD_OFFSET, self.__stylus_host.clone());
            let value = store.load();
            alias.erase();
            value
        }
    }
}

impl<K, V> StorageMap<K, V>
where
    K: StorageKey,
    V: Erase,
{
    /// Delete the element at the given key, if it exists.
    pub fn delete(&mut self, key: K) {
        let mut store = self.setter(key);
        store.erase();
    }
}

/// Trait that allows types to be the key of a [`StorageMap`].
///
/// Note: the assignment of slots must be injective.
pub trait StorageKey {
    /// Assigns a slot based on the key and where the map is rooted.
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
        let int: U160 = self.0.into();
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
    ($($uint:ident $int:ident)+) => {
        $(
            impl StorageKey for $uint {
                fn to_slot(&self, root: B256) -> U256 {
                    let data = B256::from(U256::from(*self));
                    let data = data.concat_const::<32, 64>(root.into());
                    crypto::keccak(data).into()
                }
            }

            impl StorageKey for $int {
                fn to_slot(&self, root: B256) -> U256 {
                    let data = B256::from(U256::from(*self as $uint)); // wrap-around
                    let data = data.concat_const::<32, 64>(root.into());
                    crypto::keccak(data).into()
                }
            }
        )+
    };
}

impl_key!(u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize);
