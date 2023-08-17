// Copyright 2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use super::{Erase, StorageGuard, StorageGuardMut, StorageType};
use alloy_primitives::U256;
use std::marker::PhantomData;

/// Accessor for a storage-backed array.
pub struct StorageArray<S: StorageType, const L: usize> {
    marker: PhantomData<S>,
    item_slots: Vec<U256>,
}

impl<S: StorageType, const L: usize> StorageType for StorageArray<S, L> {
    type Wraps<'a> = StorageGuard<'a, StorageArray<S, L>> where Self: 'a;
    type WrapsMut<'a> = StorageGuardMut<'a, StorageArray<S, L>> where Self: 'a;

    // Must have at least one required slot.
    const REQUIRED_SLOTS: usize = 1;

    unsafe fn new(slot: U256, offset: u8) -> Self {
        let mut curr_slot = slot;
        let mut item_slots = vec![];
        for _ in 0..L {
            // TODO: Deal with offsets properly.
            let _ = S::new(curr_slot, 0);
            curr_slot = curr_slot + alloy_primitives::U256::from(S::REQUIRED_SLOTS);
            item_slots.push(curr_slot);
        }
        debug_assert!(offset == 0);
        Self {
            marker: PhantomData,
            item_slots,
        }
    }

    fn load<'s>(self) -> Self::Wraps<'s> {
        StorageGuard::new(self)
    }

    fn load_mut<'s>(self) -> Self::WrapsMut<'s> {
        StorageGuardMut::new(self)
    }
}

impl<S: StorageType, const L: usize> StorageArray<S, L> {
    /// Gets the element at the given index, if it exists.
    pub fn get(&self, index: impl TryInto<usize>) -> Option<S::Wraps<'_>> {
        let slot = self.item_slots.get(index.try_into().ok()?)?;
        let s = unsafe { S::new(*slot, 0) };
        Some(s.load())
    }

    /// Gets a mutable accessor to the element at a given index, if it exists.
    pub fn get_mut(&mut self, index: impl TryInto<usize>) -> Option<S::WrapsMut<'_>> {
        let slot = self.item_slots.get(index.try_into().ok()?)?;
        let s = unsafe { S::new(*slot, 0) };
        Some(s.load_mut())
    }
}

impl<S: Erase, const L: usize> Erase for StorageArray<S, L> {
    fn erase(&mut self) {
        for slot in self.item_slots.iter() {
            let mut s = unsafe { S::new(*slot, 0) };
            s.erase();
        }
    }
}
