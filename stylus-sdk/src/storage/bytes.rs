// Copyright 2023, Offchain Labs, Inc.
// For license information, see https://github.com/OffchainLabs/nitro/blob/master/LICENSE

use super::{StorageCache, StorageType};
use crate::crypto;
use alloy_primitives::U256;
use std::cell::OnceCell;

/// Accessor for storage-backed bytes
pub struct StorageBytes {
    slot: U256,
    base: OnceCell<U256>,
}

impl StorageType for StorageBytes {
    fn new(slot: U256, offset: u8) -> Self {
        debug_assert!(offset == 0);
        Self {
            slot,
            base: OnceCell::new(),
        }
    }
}

impl StorageBytes {
    /// Returns `true` if the collection contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets the number of bytes stored.
    pub fn len(&self) -> usize {
        let word = StorageCache::get_word(self.slot);

        // check if the data is short
        let slot: &[u8] = word.as_ref();
        if slot[31] == 0 {
            return (slot[31] / 2) as usize;
        }

        let word: U256 = word.into();
        let len = word / U256::from(2);
        len.try_into().unwrap()
    }

    /// Adds a byte to the end.
    fn push(&mut self, b: u8) {
        todo!()
    }

    /// Determines where in storage indices start. Could be made const in the future.
    fn base(&self) -> &U256 {
        self.base
            .get_or_init(|| crypto::keccak(self.slot.to_be_bytes::<32>()).into())
    }
}
