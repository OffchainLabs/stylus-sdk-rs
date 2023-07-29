// Copyright 2023, Offchain Labs, Inc.
// For license information, see https://github.com/OffchainLabs/nitro/blob/master/LICENSE

use super::{SizedStorageType, StorageCache, StorageGuard, StorageGuardMut, StorageType};
use crate::crypto;
use alloy_primitives::U256;
use std::{cell::OnceCell, marker::PhantomData};

/// Accessor for a storage-backed vector
pub struct StorageVec<S: StorageType> {
    slot: U256,
    base: OnceCell<U256>,
    marker: PhantomData<S>,
}

impl<S: StorageType> StorageType for StorageVec<S> {
    type Wraps<'a> = StorageGuard<'a, StorageVec<S>> where Self: 'a;
    type WrapsMut<'a> = StorageGuardMut<'a, StorageVec<S>> where Self: 'a;

    unsafe fn new(slot: U256, offset: u8) -> Self {
        debug_assert!(offset == 0);
        Self {
            slot,
            base: OnceCell::new(),
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

impl<S: StorageType> StorageVec<S> {
    /// Returns `true` if the collection contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets the number of elements stored.
    pub fn len(&self) -> usize {
        let word: U256 = StorageCache::get_word(self.slot).into();
        word.try_into().unwrap()
    }

    /// Overwrites the vector's length.
    ///
    /// # Safety
    ///
    /// It must be sensible to create accessors for `S` from zero-slots,
    /// or any junk data left over from previous dirty removal operations.
    /// Note that `StorageVec` has unlimited capacity, so all lengths are valid.
    pub unsafe fn set_len(&mut self, len: usize) {
        StorageCache::set_word(self.slot, U256::from(len).into())
    }
    /// Gets an accessor to the element at a given index, if it exists.
    /// Note: the accessor is protected by a [`StoreageGuard`], which restricts
    /// its lifetime to that of `&self`.
    pub fn getter(&self, index: impl TryInto<usize>) -> Option<StorageGuard<S>> {
        let store = unsafe { self.accessor(index)? };
        Some(StorageGuard::new(store))
    }

    /// Gets a mutable accessor to the element at a given index, if it exists.
    /// Note: the accessor is protected by a [`StoreageGuardMut`], which restricts
    /// its lifetime to that of `&mut self`.
    pub fn setter(&mut self, index: impl TryInto<usize>) -> Option<StorageGuardMut<S>> {
        let store = unsafe { self.accessor(index)? };
        Some(StorageGuardMut::new(store))
    }

    /// Gets the underlying accessor to the element at a given index, if it exists.
    ///
    /// # Safety
    ///
    /// Because the accessor is unconstrained by a storage guard, storage aliasing is possible
    /// if used incorrectly. Two or more mutable references to the same `S` are possible, as are
    /// read-after-write scenarios.
    pub unsafe fn accessor(&self, index: impl TryInto<usize>) -> Option<S> {
        let index = index.try_into().ok()?;
        let width = S::SIZE as usize;

        if index > self.len() {
            return None;
        }

        let density = 32 / width;
        let offset = self.base() + U256::from(width * index / density);
        unsafe { Some(S::new(offset, (index % density) as u8)) }
    }

    /// Like [`std::Vec::push`], but returns a mutable accessor to the new slot.
    /// This enables pushing elements without constructing them first.
    ///
    /// # Example
    ///
    /// ```
    /// use stylus_sdk::storage::{StorageVec, StorageType, StorageU256};
    /// use stylus_sdk::alloy_primitives::U256;
    ///
    /// let mut vec: StorageVec<StorageVec<StorageU256>> = StorageVec::new(U256::ZERO, 0);
    /// let mut inner_vec = vec.open();
    /// inner_vec.push(U256::from(8));
    ///
    /// let value = inner_vec.get(0).unwrap();
    /// assert_eq!(value.get(), U256::from(8));
    /// assert_eq!(inner_vec.len(), 1);
    /// ```
    pub fn grow(&mut self) -> StorageGuardMut<S> {
        let index = self.len();
        let width = S::SIZE as usize;
        unsafe { self.set_len(index) };

        let density = 32 / width;
        let offset = self.base() + U256::from(width * index / density);
        let store = unsafe { S::new(offset, (index % density) as u8) };
        StorageGuardMut::new(store)
    }

    /// Removes and returns an accessor to the last element of the vector, if any.
    pub fn shrink(&mut self) -> Option<S> {
        let index = match self.len() {
            0 => return None,
            x => x - 1,
        };
        let item = unsafe { self.accessor(index) };
        StorageCache::set_word(self.slot, U256::from(index).into());
        item
    }

    /// Shortens the vector, keeping the first `len` elements.
    /// Note: this method does not clear any underlying storage.
    pub fn truncate(&mut self, len: usize) {
        if len < self.len() {
            // SAFETY: operation leaves only existing values
            unsafe { self.set_len(len) }
        }
    }

    /// Determines where in storage indices start. Could be made const in the future.
    fn base(&self) -> &U256 {
        self.base
            .get_or_init(|| crypto::keccak(self.slot.to_be_bytes::<32>()).into())
    }

    pub fn get(&self, index: impl TryInto<usize>) -> Option<S::Wraps<'_>> {
        let store = unsafe { self.accessor(index)? };
        Some(store.load())
    }

    pub fn get_mut(&mut self, index: impl TryInto<usize>) -> Option<S::WrapsMut<'_>> {
        let store = unsafe { self.accessor(index)? };
        Some(store.load_mut())
    }
}

impl<'a, S: SizedStorageType<'a>> StorageVec<S> {
    /// Adds an element to the end of the vector.
    pub fn push(&mut self, value: S::Wraps<'a>) {
        let mut store = self.grow();
        store.set_exact(value);
    }

    /// Removes and returns the last element of the vector, if any.
    /// Note: the underlying storage slot is zero'd out when all elements in the word are freed.
    pub fn pop(&mut self) -> Option<S::Wraps<'a>> {
        let store = self.shrink()?;
        let index = self.len();
        let value = store.into();
        let width = S::SIZE as usize;

        // TODO: cleanup with accessor trait
        let density = (32 / S::SIZE) as usize;
        if index % density == 0 {
            let offset = self.base() + U256::from(width * index / density);
            unsafe { S::new(offset, 0).erase() };
        }
        Some(value)
    }
}

impl<'a, S: SizedStorageType<'a>> Extend<S::Wraps<'a>> for StorageVec<S> {
    fn extend<T: IntoIterator<Item = S::Wraps<'a>>>(&mut self, iter: T) {
        for elem in iter {
            self.push(elem);
        }
    }
}
