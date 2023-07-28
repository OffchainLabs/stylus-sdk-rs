// Copyright 2023, Offchain Labs, Inc.
// For license information, see https://github.com/OffchainLabs/nitro/blob/master/LICENSE

use super::{StorageCache, StorageType};
use crate::crypto;
use alloy_primitives::{B256, U256, U8};
use std::cell::OnceCell;

/// Accessor for storage-backed bytes
pub struct StorageBytes {
    root: U256,
    base: OnceCell<U256>,
}

impl StorageType for StorageBytes {
    fn new(root: U256, offset: u8) -> Self {
        debug_assert!(offset == 0);
        Self {
            root,
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
        let word = StorageCache::get_word(self.root);

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
    pub fn push(&mut self, b: u8) {
        let index = self.len();
        let value = U8::from(b);

        if index < 31 {
            return unsafe { StorageCache::set_uint(self.root, index, value) };
        }

        // convert to multi-word representation
        if index == 31 {
            // copy content over (len byte will be overwritten)
            let word = StorageCache::get_word(self.root);
            StorageCache::set_word(*self.base(), word);

            // place the len in the root with the long bit high
            StorageCache::set_word(self.root, U256::from(32 * 2 + 1).into())
        }

        let slot = self.base() + U256::from(index / 32);
        unsafe { StorageCache::set_uint(slot, index % 32, value) };
    }

    /// Removes and returns the last byte.
    pub fn pop(&mut self) -> Option<u8> {
        let len = self.len();
        if len == 0 {
            return None;
        }

        let index = len - 1;
        let clean = index % 32 == 0;

        if len > 32 {
            let slot = self.base() + U256::from(index / 32);
            let byte = unsafe { StorageCache::get_byte(slot, index % 32) };

            // place the len in the root with the long bit high
            let len = U256::from(len * 2 + 1);
            StorageCache::set_word(self.root, len.into());

            if clean {
                StorageCache::set_word(slot, B256::ZERO);
            }
            return Some(byte);
        }

        if len == 32 {}

        todo!()
    }

    /// Determines where in storage indices start. Could be made const in the future.
    fn base(&self) -> &U256 {
        self.base
            .get_or_init(|| crypto::keccak(self.root.to_be_bytes::<32>()).into())
    }
}

// TODO: efficient bulk insertion
impl Extend<u8> for StorageBytes {
    fn extend<T: IntoIterator<Item = u8>>(&mut self, iter: T) {
        for elem in iter {
            self.push(elem);
        }
    }
}

/// Accessor for storage-backed bytes
pub struct StorageString(pub StorageBytes);

impl StorageType for StorageString {
    fn new(slot: U256, offset: u8) -> Self {
        Self(StorageBytes::new(slot, offset))
    }
}

impl StorageString {
    /// Returns `true` if the collection contains no elements.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Gets the number of bytes stored.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn push(&mut self, c: char) {
        for byte in c.to_string().bytes() {
            self.0.push(byte)
        }
    }
}

impl Extend<char> for StorageString {
    fn extend<T: IntoIterator<Item = char>>(&mut self, iter: T) {
        for c in iter {
            self.push(c);
        }
    }
}
