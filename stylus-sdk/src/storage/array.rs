// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use super::{Erase, StorageGuard, StorageGuardMut, StorageType};
use alloy_primitives::U256;
use core::marker::PhantomData;

/// Accessor for a storage-backed array.
pub struct StorageArray<S: StorageType, const N: usize> {
    slot: U256,
    marker: PhantomData<S>,
}

impl<S: StorageType, const N: usize> StorageType for StorageArray<S, N> {
    type Wraps<'a> = StorageGuard<'a, StorageArray<S, N>> where Self: 'a;
    type WrapsMut<'a> = StorageGuardMut<'a, StorageArray<S, N>> where Self: 'a;

    const REQUIRED_SLOTS: usize = Self::required_slots();

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

impl<S: StorageType, const N: usize> StorageArray<S, N> {
    /// Gets the number of elements stored.
    ///
    /// Although this type will always have the same length, this method is still provided for
    /// consistency with [`StorageVec`].
    pub const fn len(&self) -> usize {
        N
    }

    /// Gets an accessor to the element at a given index, if it exists.
    /// Note: the accessor is protected by a [`StorageGuard`], which restricts
    /// its lifetime to that of `&self`.
    pub fn getter(&self, index: impl TryInto<usize>) -> Option<StorageGuard<S>> {
        let store = unsafe { self.accessor(index)? };
        Some(StorageGuard::new(store))
    }

    /// Gets a mutable accessor to the element at a given index, if it exists.
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
        if index >= N {
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

    /// Determines the slot and offset for the element at an index.
    fn index_slot(&self, index: usize) -> (U256, u8) {
        let width = S::SLOT_BYTES;
        let words = S::REQUIRED_SLOTS.max(1);
        let density = Self::density();

        let slot = self.slot + U256::from(words * index / density);
        let offset = 32 - (width * (1 + index % density)) as u8;
        (slot, offset)
    }

    /// Number of elements per slot.
    const fn density() -> usize {
        32 / S::SLOT_BYTES
    }

    /// Required slots for the storage array.
    const fn required_slots() -> usize {
        let reserved = N * S::REQUIRED_SLOTS;
        let density = Self::density();
        let packed = (N + density - 1) / density; // ceil division for packed items.
        if reserved > packed {
            return reserved;
        }
        packed
    }
}

impl<S: Erase, const N: usize> Erase for StorageArray<S, N> {
    fn erase(&mut self) {
        for i in 0..N {
            let mut store = unsafe { self.accessor_unchecked(i) };
            store.erase()
        }
    }
}
