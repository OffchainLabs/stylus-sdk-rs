// Copyright 2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use super::{
    Erase, GlobalStorage, SimpleStorageType, Storage, StorageGuard, StorageGuardMut, StorageType,
};
use crate::crypto;
use alloy_primitives::U256;
use core::{cell::OnceCell, marker::PhantomData};

/// Accessor for a storage-backed vector.
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
        let word: U256 = Storage::get_word(self.slot).into();
        word.try_into().unwrap()
    }

    /// Overwrites the vector's length.
    ///
    /// # Safety
    ///
    /// It must be sensible to create accessors for `S` from zero-slots,
    /// or any junk data left over from prior dirty operations.
    /// Note that [`StorageVec`] has unlimited capacity, so all lengths are valid.
    pub unsafe fn set_len(&mut self, len: usize) {
        Storage::set_word(self.slot, U256::from(len).into())
    }

    /// Gets an accessor to the element at a given index, if it exists.
    ///
    /// Note: the accessor is protected by a [`StorageGuard`], which restricts
    /// its lifetime to that of `&self`.
    pub fn getter(&self, index: impl TryInto<usize>) -> Option<StorageGuard<S>> {
        let store = unsafe { self.accessor(index)? };
        Some(StorageGuard::new(store))
    }

    /// Gets a mutable accessor to the element at a given index, if it exists.
    ///
    /// Note: the accessor is protected by a [`StorageGuardMut`], which restricts
    /// its lifetime to that of `&mut self`.
    pub fn setter(&mut self, index: impl TryInto<usize>) -> Option<StorageGuardMut<S>> {
        let store = unsafe { self.accessor(index)? };
        Some(StorageGuardMut::new(store))
    }

    /// Gets the underlying accessor to the element at a given index, if it exists.
    ///
    /// # Safety
    ///
    /// Enables aliasing.
    unsafe fn accessor(&self, index: impl TryInto<usize>) -> Option<S> {
        let index = index.try_into().ok()?;
        if index >= self.len() {
            return None;
        }
        let (slot, offset) = self.index_slot(index);
        Some(S::new(slot, offset))
    }

    /// Gets the underlying accessor to the element at a given index, even if out of bounds.
    ///
    /// # Safety
    ///
    /// Enables aliasing. UB if out of bounds.
    unsafe fn accessor_unchecked(&self, index: usize) -> S {
        let (slot, offset) = self.index_slot(index);
        S::new(slot, offset)
    }

    /// Gets the element at the given index, if it exists.
    pub fn get(&self, index: impl TryInto<usize>) -> Option<S::Wraps<'_>> {
        let store = unsafe { self.accessor(index)? };
        Some(store.load())
    }

    /// Gets a mutable accessor to the element at a given index, if it exists.
    pub fn get_mut(&mut self, index: impl TryInto<usize>) -> Option<S::WrapsMut<'_>> {
        let store = unsafe { self.accessor(index)? };
        Some(store.load_mut())
    }

    /// Like [`std::vec::Vec::push`][vec_push], but returns a mutable accessor to the new slot.
    /// This enables pushing elements without constructing them first.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use stylus_sdk::storage::{StorageVec, StorageType, StorageU256};
    /// use stylus_sdk::alloy_primitives::U256;
    ///
    /// let mut vec: StorageVec<StorageVec<StorageU256>> = unsafe { StorageVec::new(U256::ZERO, 0) };
    /// let mut inner_vec = vec.grow();
    /// inner_vec.push(U256::from(8));
    ///
    /// let value = inner_vec.get(0).unwrap();
    /// assert_eq!(value, U256::from(8));
    /// assert_eq!(inner_vec.len(), 1);
    /// ```
    ///
    /// [vec_push]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.push
    pub fn grow(&mut self) -> StorageGuardMut<S> {
        let index = self.len();
        unsafe { self.set_len(index + 1) };

        let (slot, offset) = self.index_slot(index);
        let store = unsafe { S::new(slot, offset) };
        StorageGuardMut::new(store)
    }

    /// Removes and returns an accessor to the last element of the vector, if any.
    pub fn shrink(&mut self) -> Option<StorageGuardMut<S>> {
        let index = match self.len() {
            0 => return None,
            x => x - 1,
        };
        unsafe {
            self.set_len(index);
            Some(StorageGuardMut::new(self.accessor_unchecked(index)))
        }
    }

    /// Shortens the vector, keeping the first `len` elements.
    ///
    /// Note: this method does not erase any underlying storage.
    pub fn truncate(&mut self, len: usize) {
        if len < self.len() {
            // SAFETY: operation leaves only existing values
            unsafe { self.set_len(len) }
        }
    }

    /// Determines the slot and offset for the element at an index.
    fn index_slot(&self, index: usize) -> (U256, u8) {
        let width = S::SLOT_BYTES;
        let words = S::REQUIRED_SLOTS.max(1);
        let density = self.density();

        let slot = self.base() + U256::from(words * index / density);
        let offset = 32 - (width * (1 + index % density)) as u8;
        (slot, offset)
    }

    /// Number of elements per slot.
    const fn density(&self) -> usize {
        32 / S::SLOT_BYTES
    }

    /// Determines where in storage indices start. Could be made `const` in the future.
    fn base(&self) -> &U256 {
        self.base
            .get_or_init(|| crypto::keccak(self.slot.to_be_bytes::<32>()).into())
    }
}

impl<'a, S: SimpleStorageType<'a>> StorageVec<S> {
    /// Adds an element to the end of the vector.
    pub fn push(&mut self, value: S::Wraps<'a>) {
        let mut store = self.grow();
        store.set_by_wrapped(value);
    }

    /// Removes and returns the last element of the vector, if it exists.
    ///
    /// Note: the underlying storage slot is erased when all elements in a word are freed.
    pub fn pop(&mut self) -> Option<S::Wraps<'a>> {
        let store = unsafe { self.shrink()?.into_raw() };
        let index = self.len();
        let value = store.into();
        let first = index % self.density() == 0;

        if first {
            let slot = self.index_slot(index).0;
            let words = S::REQUIRED_SLOTS.max(1);
            for i in 0..words {
                unsafe { Storage::clear_word(slot + U256::from(i)) };
            }
        }
        Some(value)
    }
}

impl<S: Erase> StorageVec<S> {
    /// Removes and erases the last element of the vector.
    pub fn erase_last(&mut self) {
        if self.is_empty() {
            return;
        }
        let index = self.len() - 1;
        unsafe {
            self.accessor_unchecked(index).erase();
            self.set_len(index);
        }
    }
}

impl<S: Erase> Erase for StorageVec<S> {
    fn erase(&mut self) {
        for i in 0..self.len() {
            let mut store = unsafe { self.accessor_unchecked(i) };
            store.erase()
        }
        self.truncate(0);
    }
}

impl<'a, S: SimpleStorageType<'a>> Extend<S::Wraps<'a>> for StorageVec<S> {
    fn extend<T: IntoIterator<Item = S::Wraps<'a>>>(&mut self, iter: T) {
        for elem in iter {
            self.push(elem);
        }
    }
}
