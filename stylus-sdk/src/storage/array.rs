// Copyright 2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use super::{Erase, StorageGuard, StorageGuardMut, StorageType};
use alloy_primitives::U256;
use std::marker::PhantomData;

/// Accessor for a storage-backed array.
pub struct StorageArray<S: StorageType, const N: usize> {
    slot: U256,
    marker: PhantomData<S>,
}

impl<S: StorageType, const N: usize> StorageType for StorageArray<S, N> {
    type Wraps<'a> = StorageGuard<'a, StorageArray<S, N>> where Self: 'a;
    type WrapsMut<'a> = StorageGuardMut<'a, StorageArray<S, N>> where Self: 'a;

    // Because certain types, such as some primitives, can have 0 required slots
    // (many of them can fit in a single slot), we need to do a computation
    // to figure out how many slots in total the array will take up.
    // For example, if we have an element that takes up 8 bytes, and we want
    // a fixed array of 10 of these elements, we will need 80 bytes in total,
    // which would fit into 3 slots.
    const REQUIRED_SLOTS: usize = Self::required_slots();

    unsafe fn new(slot: U256, offset: u8) -> Self {
        debug_assert!(offset == 0);
        debug_assert!(N > 0);
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
        let density = self.density();

        let slot = self.slot + U256::from(words * index / density);
        let offset = 32 - (width * (1 + index % density)) as u8;
        (slot, offset)
    }

    /// Number of elements per slot.
    const fn density(&self) -> usize {
        32 / S::SLOT_BYTES
    }

    /// Required slots for the storage array. A maximum of either N * S::REQUIRED_SLOTS,
    /// or ceil((S::SLOT_BYTES * N) / 32), as there are items that can fit multiple times
    /// in a single slot.
    const fn required_slots() -> usize {
        let left = N * S::REQUIRED_SLOTS;
        let right = ceil_div(S::SLOT_BYTES * N, 32);
        if left > right {
            return left;
        }
        right
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

// Note: b must be non-zero.
const fn ceil_div(a: usize, b: usize) -> usize {
    (a + (b - 1)) / b
}

#[cfg(test)]
mod test {
    use super::ceil_div;

    #[test]
    fn test_ceil() {
        assert_eq!(ceil_div(80, 32), 3);
        assert_eq!(ceil_div(1, 1), 1);
        assert_eq!(ceil_div(0, 1), 0);
        assert_eq!(ceil_div(100, 30), 4);
    }
}
