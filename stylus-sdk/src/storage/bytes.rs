// Copyright 2022-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use super::{Erase, GlobalStorage, Storage, StorageB8, StorageGuard, StorageGuardMut, StorageType};
use crate::crypto;
use crate::host::VM;
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use alloy_primitives::{U256, U8};
use cfg_if::cfg_if;
use core::cell::OnceCell;
use stylus_core::HostAccess;
use stylus_test::mock::TestHost;

/// Accessor for storage-backed bytes.
pub struct StorageBytes {
    root: U256,
    base: OnceCell<U256>,
    __stylus_host: VM,
}

impl StorageType for StorageBytes {
    type Wraps<'a>
        = StorageGuard<'a, StorageBytes>
    where
        Self: 'a;
    type WrapsMut<'a>
        = StorageGuardMut<'a, StorageBytes>
    where
        Self: 'a;

    unsafe fn new(root: U256, offset: u8, host: VM) -> Self {
        debug_assert!(offset == 0);
        Self {
            root,
            base: OnceCell::new(),
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

impl HostAccess for StorageBytes {
    fn vm(&self) -> &dyn stylus_core::Host {
        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                &self.__stylus_host
            } else {
                unsafe {
                    core::mem::transmute::<&dyn TestHost, &dyn stylus_core::Host>(&**self.__stylus_host.host)
                }
            }
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
        let word = Storage::get_word(self.__stylus_host.clone(), self.root);

        // check if the data is short
        let slot: &[u8] = word.as_ref();
        if slot[31] & 1 == 0 {
            return (slot[31] / 2) as usize;
        }

        let word: U256 = word.into();
        let len = word / U256::from(2);
        len.try_into().unwrap()
    }

    /// Overwrites the collection's length, moving bytes as needed.
    ///
    /// # Safety
    ///
    /// May populate the vector with junk bytes from prior dirty operations.
    /// Note that [`StorageBytes`] has unlimited capacity, so all lengths are valid.
    pub unsafe fn set_len(&mut self, len: usize) {
        let old = self.len();

        // if representation hasn't changed, just update the length
        if (old < 32) == (len < 32) {
            return self.write_len(len);
        }

        // if shrinking, pull data in
        if (len < 32) && (old > 32) {
            let word = Storage::get_word(self.__stylus_host.clone(), *self.base());
            Storage::set_word(self.__stylus_host.clone(), self.root, word);
            return self.write_len(len);
        }

        // if growing, push data out
        let mut word = Storage::get_word(self.__stylus_host.clone(), self.root);
        word[31] = 0; // clear len byte
        Storage::set_word(self.__stylus_host.clone(), *self.base(), word);
        self.write_len(len)
    }

    /// Updates the length while being conscious of representation.
    unsafe fn write_len(&mut self, len: usize) {
        if len < 32 {
            // place the len in the last byte of the root with the long bit low
            Storage::set_uint(self.__stylus_host.clone(), self.root, 31, U8::from(len * 2));
        } else {
            // place the len in the root with the long bit high
            Storage::set_word(
                self.__stylus_host.clone(),
                self.root,
                U256::from(len * 2 + 1).into(),
            )
        }
    }

    /// Adds a byte to the end.
    pub fn push(&mut self, b: u8) {
        let index = self.len();
        let value = U8::from(b);

        macro_rules! assign {
            ($slot:expr) => {
                unsafe {
                    Storage::set_uint(self.__stylus_host.clone(), $slot, index % 32, value); // pack value
                    self.write_len(index + 1);
                }
            };
        }

        if index < 31 {
            return assign!(self.root);
        }

        // convert to multi-word representation
        if index == 31 {
            // copy content over (len byte will be overwritten)
            let word = Storage::get_word(self.__stylus_host.clone(), self.root);
            unsafe { Storage::set_word(self.__stylus_host.clone(), *self.base(), word) };
        }

        let slot = self.base() + U256::from(index / 32);
        assign!(slot);
    }

    /// Removes and returns the last byte, if it exists.
    /// As an optimization, underlying storage slots are only erased when all bytes in
    /// a given word are freed when in the multi-word representation.
    pub fn pop(&mut self) -> Option<u8> {
        let len = self.len();
        if len == 0 {
            return None;
        }

        let index = len - 1;
        let clean = index % 32 == 0;
        let byte = self.get(index)?;

        let clear = |slot| unsafe { Storage::clear_word(self.__stylus_host.clone(), slot) };

        // convert to single-word representation
        if len == 32 {
            // copy content over
            let word = Storage::get_word(self.__stylus_host.clone(), *self.base());
            unsafe { Storage::set_word(self.__stylus_host.clone(), self.root, word) };
            clear(*self.base());
        }

        // clear distant word
        if len > 32 && clean {
            clear(self.index_slot(len - 1).0);
        }

        // clear the value
        if len < 32 {
            unsafe { Storage::set_byte(self.__stylus_host.clone(), self.root, index, 0) };
        }

        // set the new length
        unsafe { self.write_len(index) };
        Some(byte)
    }

    /// Gets the byte at the given index, if it exists.
    pub fn get(&self, index: impl TryInto<usize>) -> Option<u8> {
        let index = index.try_into().ok()?;
        if index >= self.len() {
            return None;
        }
        unsafe { Some(self.get_unchecked(index)) }
    }

    /// Gets a mutable accessor to the byte at the given index, if it exists.
    pub fn get_mut(&mut self, index: impl TryInto<usize>) -> Option<StorageGuardMut<StorageB8>> {
        let index = index.try_into().ok()?;
        if index >= self.len() {
            return None;
        }
        let (slot, offset) = self.index_slot(index);
        let value = unsafe { StorageB8::new(slot, offset, self.__stylus_host.clone()) };
        Some(StorageGuardMut::new(value))
    }

    /// Gets the byte at the given index, even if beyond the collection.
    ///
    /// # Safety
    ///
    /// UB if index is out of bounds.
    pub unsafe fn get_unchecked(&self, index: usize) -> u8 {
        let (slot, offset) = self.index_slot(index);
        unsafe { Storage::get_byte(self.__stylus_host.clone(), slot, offset.into()) }
    }

    /// Gets the full contents of the collection.
    pub fn get_bytes(&self) -> Vec<u8> {
        let len = self.len();
        let mut bytes = Vec::with_capacity(len);

        for i in 0..len {
            let byte = unsafe { self.get_unchecked(i) };
            bytes.push(byte);
        }
        bytes
    }

    /// Overwrites the contents of the collection, erasing what was previously stored.
    pub fn set_bytes(&mut self, bytes: impl AsRef<[u8]>) {
        self.erase();
        self.extend(bytes.as_ref());
    }

    /// Determines the slot and offset for the element at an index.
    fn index_slot(&self, index: usize) -> (U256, u8) {
        let slot = match self.len() {
            32.. => self.base() + U256::from(index / 32),
            _ => self.root,
        };
        (slot, (index % 32) as u8)
    }

    /// Determines where in storage indices start. Could be made `const` in the future.
    fn base(&self) -> &U256 {
        self.base
            .get_or_init(|| crypto::keccak(self.root.to_be_bytes::<32>()).into())
    }
}

impl Erase for StorageBytes {
    fn erase(&mut self) {
        let mut len = self.len() as isize;
        if len > 31 {
            while len > 0 {
                let slot = self.index_slot(len as usize - 1).0;
                unsafe { Storage::clear_word(self.__stylus_host.clone(), slot) };
                len -= 32;
            }
        }
        unsafe { Storage::clear_word(self.__stylus_host.clone(), self.root) };
    }
}

impl Extend<u8> for StorageBytes {
    fn extend<T: IntoIterator<Item = u8>>(&mut self, iter: T) {
        for elem in iter {
            self.push(elem);
        }
    }
}

impl<'a> Extend<&'a u8> for StorageBytes {
    fn extend<T: IntoIterator<Item = &'a u8>>(&mut self, iter: T) {
        for elem in iter {
            self.push(*elem);
        }
    }
}

/// Accessor for storage-backed bytes
pub struct StorageString(pub StorageBytes);

impl StorageType for StorageString {
    type Wraps<'a>
        = StorageGuard<'a, StorageString>
    where
        Self: 'a;
    type WrapsMut<'a>
        = StorageGuardMut<'a, StorageString>
    where
        Self: 'a;

    unsafe fn new(slot: U256, offset: u8, host: VM) -> Self {
        Self(StorageBytes::new(slot, offset, host))
    }

    fn load<'s>(self) -> Self::Wraps<'s> {
        StorageGuard::new(self)
    }

    fn load_mut<'s>(self) -> Self::WrapsMut<'s> {
        StorageGuardMut::new(self)
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

    /// Adds a char to the end.
    pub fn push(&mut self, c: char) {
        for byte in c.to_string().bytes() {
            self.0.push(byte)
        }
    }

    /// Gets the underlying [`String`], ignoring any invalid data.
    pub fn get_string(&self) -> String {
        let bytes = self.0.get_bytes();
        String::from_utf8_lossy(&bytes).into()
    }

    /// Overwrites the underlying [`String`], erasing what was previously stored.
    pub fn set_str(&mut self, text: impl AsRef<str>) {
        self.erase();
        for c in text.as_ref().chars() {
            self.push(c);
        }
    }
}

impl Erase for StorageString {
    fn erase(&mut self) {
        self.0.erase()
    }
}

impl Extend<char> for StorageString {
    fn extend<T: IntoIterator<Item = char>>(&mut self, iter: T) {
        for c in iter {
            self.push(c);
        }
    }
}
