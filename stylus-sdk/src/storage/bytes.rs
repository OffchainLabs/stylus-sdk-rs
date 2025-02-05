// Copyright 2022-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use super::{Erase, GlobalStorage, Storage, StorageB8, StorageGuard, StorageGuardMut, StorageType};
use crate::crypto;
use crate::host::VM;
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use alloy_primitives::{B256, U256, U8};
use cfg_if::cfg_if;
use core::{borrow::Borrow, cell::OnceCell};
use stylus_core::HostAccess;

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
                self.__stylus_host.host.as_ref()
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<T> From<&T> for StorageBytes
where
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

impl StorageBytes {
    /// Returns `true` if the collection contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets the number of bytes stored.
    pub fn len(&self) -> usize {
        BytesRoot::new(self).len()
    }

    /// Overwrites the collection's length, moving bytes as needed.
    ///
    /// # Safety
    ///
    /// May populate the vector with junk bytes from prior dirty operations.
    /// Note that [`StorageBytes`] has unlimited capacity, so all lengths are valid.
    pub unsafe fn set_len(&mut self, len: usize) {
        let mut root = BytesRoot::new_mut(self);
        let old = root.len();

        // if representation hasn't changed, just update the length
        if (old < 32) == (len < 32) {
            return root.write_len(len);
        }

        // if shrinking, pull data in
        if (len < 32) && (old >= 32) {
            root.word = Storage::get_word(root.storage.clone_vm(), *root.storage.base());
            return root.write_len(len);
        }

        // if growing, push data out
        root.word[31] = 0; // clear len byte
        Storage::set_word(root.storage.clone_vm(), *root.storage.base(), root.word);
        root.write_len(len);
    }

    /// Adds a byte to the end.
    pub fn push(&mut self, value: u8) {
        let mut root = BytesRoot::new_mut(self);
        let index = root.len();

        // still short representation after adding a byte
        // add the byte and update length
        if index < 31 {
            root.word[index] = value;
            unsafe {
                return root.write_len(index + 1);
            }
        }

        // convert to multi-word representation
        if index == 31 {
            // copy content over (len byte will be overwritten)
            root.word[index] = value;
            unsafe {
                Storage::set_word(root.storage.clone_vm(), *root.storage.base(), root.word);
                return root.write_len(index + 1);
            }
        }

        // already long representation
        // add the new byte and update length
        let (slot, offset) = root.index_slot(index);
        unsafe {
            Storage::set_uint(
                root.storage.clone_vm(),
                slot,
                offset as usize,
                U8::from(value),
            );
            root.write_len(index + 1);
        }
    }

    /// Removes and returns the last byte, if it exists.
    /// As an optimization, underlying storage slots are only erased when all bytes in
    /// a given word are freed when in the multi-word representation.
    pub fn pop(&mut self) -> Option<u8> {
        let mut root = BytesRoot::new_mut(self);
        let len = root.len();
        if len == 0 {
            return None;
        }

        let index = len - 1;

        // convert to single-word representation
        if len == 32 {
            // copy content over
            let base = *root.storage.base();
            root.word = Storage::get_word(root.storage.clone_vm(), base);
            let byte = root.word[index];
            unsafe {
                root.write_len(index);
                Storage::clear_word(root.storage.clone_vm(), base);
            }
            return Some(byte);
        }

        let byte = root.get(index)?;
        let clean = index % 32 == 0;

        // clear distant word
        if len > 32 && clean {
            unsafe {
                Storage::clear_word(root.storage.clone_vm(), root.index_slot(len - 1).0);
            }
        }

        // clear the value
        if len < 32 {
            root.word[index] = 0;
        }

        // set the new length
        unsafe { root.write_len(index) };
        Some(byte)
    }

    /// Gets the byte at the given index, if it exists.
    pub fn get(&self, index: impl TryInto<usize>) -> Option<u8> {
        BytesRoot::new(self).get(index)
    }

    /// Gets a mutable accessor to the byte at the given index, if it exists.
    pub fn get_mut(&mut self, index: impl TryInto<usize>) -> Option<StorageGuardMut<StorageB8>> {
        let root = BytesRoot::new_mut(self);
        let index = index.try_into().ok()?;
        if index >= root.len() {
            return None;
        }
        let (slot, offset) = root.index_slot(index);
        let value = unsafe { StorageB8::new(slot, offset, self.clone_vm()) };
        Some(StorageGuardMut::new(value))
    }

    /// Gets the byte at the given index, even if beyond the collection.
    ///
    /// # Safety
    ///
    /// UB if index is out of bounds.
    pub unsafe fn get_unchecked(&self, index: usize) -> u8 {
        BytesRoot::new(self).get_unchecked(index)
    }

    /// Gets the full contents of the collection.
    pub fn get_bytes(&self) -> Vec<u8> {
        let root = BytesRoot::new(self);
        let len = root.len();
        let mut bytes = Vec::with_capacity(len);

        // for short representation, use appropriate number of bytes from root
        if len < 32 {
            bytes.extend_from_slice(&root.word[..len]);
            return bytes;
        }
        // for long representation, read one word at a time from storage
        for idx in (0..len).step_by(32) {
            let (slot, _) = root.index_slot(idx);
            let word = Storage::get_word(root.storage.clone_vm(), slot);
            if idx + 32 <= len {
                // entire word is part of the byte array
                bytes.extend(word.0);
            } else {
                // for the last word, only get remaining bytes
                bytes.extend(&word.0[..len - idx]);
            };
        }
        bytes
    }

    /// Overwrites the contents of the collection, erasing what was previously stored.
    pub fn set_bytes(&mut self, bytes: impl AsRef<[u8]>) {
        self.erase();
        self.extend(bytes.as_ref());
    }

    /// Returns a copy of VM.
    fn clone_vm(&self) -> VM {
        self.__stylus_host.clone()
    }

    /// Determines where in storage indices start. Could be made `const` in the future.
    fn base(&self) -> &U256 {
        self.base
            .get_or_init(|| crypto::keccak(self.root.to_be_bytes::<32>()).into())
    }
}

impl Erase for StorageBytes {
    fn erase(&mut self) {
        let root = BytesRoot::new(self);
        let mut len = root.len() as isize;
        // clear any slots used in long storage
        if len > 31 {
            while len > 0 {
                let slot = root.index_slot(len as usize - 1).0;
                unsafe { Storage::clear_word(root.storage.clone_vm(), slot) };
                len -= 32;
            }
        }
        // set length and data in root storage to zero
        unsafe { Storage::clear_word(root.storage.clone_vm(), root.storage.root) };
    }
}

impl Extend<u8> for StorageBytes {
    fn extend<T: IntoIterator<Item = u8>>(&mut self, iter: T) {
        let mut iter = iter.into_iter().peekable();
        if iter.peek().is_none() {
            return;
        }

        let mut root = BytesRoot::new_mut(self);
        let mut len = root.len();

        // if storage is small, grow it until it reaches 32 bytes
        if len < 32 {
            while len < 32 {
                if let Some(byte) = iter.next() {
                    root.word[len] = byte;
                    len += 1;
                } else {
                    break;
                }
            }
            // if storage is still small, store short representation
            if len < 32 {
                unsafe {
                    return root.write_len(len);
                }
            }
            // if len reaches 32 bytes, grow into big representation
            unsafe {
                Storage::set_word(root.storage.clone_vm(), *root.storage.base(), root.word);
            }
        }
        // we want to work with word-aligned chunks, fill in first chunk to get there
        else if len % 32 != 0 {
            let (slot, _) = root.index_slot(len - 1);
            let mut word = Storage::get_word(root.storage.clone_vm(), slot);
            while len % 32 != 0 {
                if let Some(byte) = iter.next() {
                    word[len % 32] = byte;
                    len += 1;
                } else {
                    break;
                }
            }
            // write the word we just filled in
            unsafe {
                Storage::set_word(root.storage.clone_vm(), slot, word);
            }
            // stop if iter is complete.
            if len % 32 != 0 {
                unsafe {
                    return root.write_len(len);
                }
            }
        }

        // get the slot where we will write the first chunk; we can't use root.get_slot because len is not set
        let mut slot = *root.storage.base() + U256::from(len / 32);
        let mut chunk = Vec::with_capacity(32);

        // write to storage, a word at a time
        for byte in iter {
            chunk.push(byte);
            if chunk.len() == 32 {
                unsafe {
                    Storage::set_word(root.storage.clone_vm(), slot, B256::from_slice(&chunk));
                }
                chunk.clear();
                len += 32;
                slot += U256::from(1);
            }
        }

        // write remaining chunk
        if !chunk.is_empty() {
            unsafe {
                Storage::set_word(
                    root.storage.clone_vm(),
                    slot,
                    B256::right_padding_from(&chunk),
                );
            }
            len += chunk.len();
        }

        unsafe {
            root.write_len(len);
        }
    }
}

impl<'a> Extend<&'a u8> for StorageBytes {
    fn extend<T: IntoIterator<Item = &'a u8>>(&mut self, iter: T) {
        self.extend(iter.into_iter().cloned());
    }
}

/// Contains methods to manipulate the root storage slot of StorageBytes.
struct BytesRoot<T> {
    storage: T,
    word: B256,
}

impl<T: Borrow<StorageBytes>> BytesRoot<T> {
    fn new(storage: T) -> Self {
        let vm = storage.borrow().clone_vm();
        let word = Storage::get_word(vm, storage.borrow().root);
        Self { storage, word }
    }

    fn len(&self) -> usize {
        // check if the data is short
        let slot: &[u8] = self.word.as_ref();
        if slot[31] & 1 == 0 {
            return (slot[31] / 2) as usize;
        }
        let word: U256 = self.word.into();
        let len = word / U256::from(2);
        len.try_into().unwrap()
    }

    /// Gets the byte at the given index, if it exists.
    pub fn get(&self, index: impl TryInto<usize>) -> Option<u8> {
        let index = index.try_into().ok()?;
        if index >= self.len() {
            return None;
        }
        unsafe { Some(self.get_unchecked(index)) }
    }

    /// Gets the byte at the given index, even if beyond the collection.
    ///
    /// # Safety
    ///
    /// UB if index is out of bounds.
    pub unsafe fn get_unchecked(&self, index: usize) -> u8 {
        let (slot, offset) = self.index_slot(index);
        unsafe { Storage::get_byte(self.storage.borrow().clone_vm(), slot, offset.into()) }
    }

    /// Determines the slot and offset for the element at an index.
    fn index_slot(&self, index: usize) -> (U256, u8) {
        let storage = self.storage.borrow();
        let slot = if self.len() >= 32 {
            storage.base() + U256::from(index / 32)
        } else {
            storage.root
        };
        (slot, (index % 32) as u8)
    }
}

impl<'a> BytesRoot<&'a mut StorageBytes> {
    fn new_mut(storage: &'a mut StorageBytes) -> Self {
        let word = Storage::get_word(storage.clone_vm(), storage.root);
        Self { storage, word }
    }

    unsafe fn write_len(&mut self, len: usize) {
        if len < 32 {
            self.word[31] = len as u8 * 2;
        } else {
            self.word = U256::from(len * 2 + 1).into();
        }
        Storage::set_word(self.storage.clone_vm(), self.storage.root, self.word);
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

impl HostAccess for StorageString {
    fn vm(&self) -> &dyn stylus_core::Host {
        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                &self.0.__stylus_host
            } else {
                self.0.__stylus_host.host.as_ref()
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<T> From<&T> for StorageString
where
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
        self.0.extend(text.as_ref().bytes());
    }
}

impl Erase for StorageString {
    fn erase(&mut self) {
        self.0.erase()
    }
}

impl Extend<char> for StorageString {
    fn extend<T: IntoIterator<Item = char>>(&mut self, iter: T) {
        let s = iter.into_iter().collect::<String>();
        self.0.extend(s.bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::B256;
    use stylus_test::vm::*;

    #[test]
    fn test_storage_bytes_is_empty() {
        let test_vm = TestVM::new();
        let storage = StorageBytes::from(&test_vm);

        let cases = vec![
            (B256::ZERO, true),
            (U256::from(5 * 2).into(), false),
            (U256::from(500 * 2 + 1).into(), false),
        ];

        for (value, is_empty) in cases {
            test_vm.set_storage(U256::ZERO, value);
            assert_eq!(storage.is_empty(), is_empty);
        }
    }

    #[test]
    fn test_storage_bytes_len() {
        let test_vm = TestVM::new();
        let storage = StorageBytes::from(&test_vm);

        let cases = vec![
            (B256::ZERO, 0),
            (U256::from(5 * 2).into(), 5),
            (U256::from(500 * 2 + 1).into(), 500),
        ];

        for (value, len) in cases {
            test_vm.set_storage(U256::ZERO, value);
            assert_eq!(storage.len(), len);
        }
    }

    #[test]
    fn test_storage_bytes_write_len() {
        let test_vm = TestVM::new();
        let mut storage = StorageBytes::from(&test_vm);
        let mut root = BytesRoot::new_mut(&mut storage);

        let cases = vec![
            (0, B256::ZERO),
            (5, U256::from(5 * 2).into()),
            (500, U256::from(500 * 2 + 1).into()),
        ];

        for (len, want) in cases {
            unsafe {
                root.write_len(len);
            }
            assert_eq!(test_vm.get_storage(U256::ZERO), want);
        }
    }

    #[test]
    fn test_storage_bytes_set_len_small() {
        let test_vm = TestVM::new();
        let mut storage = StorageBytes::from(&test_vm);

        let mut value = B256::right_padding_from(&[1, 2, 3, 4, 5]);
        value[31] = 5 * 2;
        test_vm.set_storage(U256::ZERO, value);

        unsafe {
            storage.set_len(4);
        }

        let mut want = value;
        want[31] = 4 * 2;
        assert_eq!(test_vm.get_storage(U256::ZERO), want);
    }

    #[test]
    fn test_storage_bytes_set_len_shrinking() {
        let test_vm = TestVM::new();
        let mut storage = StorageBytes::from(&test_vm);

        let value: B256 = U256::from(32 * 2 + 1).into();
        test_vm.set_storage(U256::ZERO, value);

        let value = B256::from([0xfa; 32]);
        test_vm.set_storage(*storage.base(), value);

        unsafe {
            storage.set_len(31);
        }

        let mut want = B256::from([0xfa; 32]);
        want[31] = 31 * 2;
        assert_eq!(test_vm.get_storage(U256::ZERO), want);
    }

    #[test]
    fn test_storage_bytes_set_len_growing() {
        let test_vm = TestVM::new();
        let mut storage = StorageBytes::from(&test_vm);

        let mut value = B256::from([0xfa; 32]);
        value[31] = 31 * 2;
        test_vm.set_storage(U256::ZERO, value);

        unsafe {
            storage.set_len(32);
        }

        let want: B256 = U256::from(32 * 2 + 1).into();
        assert_eq!(test_vm.get_storage(U256::ZERO), want);
        let want = B256::right_padding_from(&[0xfa; 31]);
        assert_eq!(test_vm.get_storage(*storage.base()), want);
    }

    #[test]
    fn test_storage_bytes_push_small() {
        let test_vm = TestVM::new();
        let mut storage = StorageBytes::from(&test_vm);

        let mut want = B256::ZERO;
        for i in 0..31 {
            storage.push(i);
            want[i as usize] = i;
            want[31] = (i + 1) * 2;
            assert_eq!(test_vm.get_storage(U256::ZERO), want);
        }
    }

    #[test]
    fn test_storage_bytes_push_big() {
        let test_vm = TestVM::new();
        let mut storage = StorageBytes::from(&test_vm);

        let value: B256 = U256::from(32 * 2 + 1).into(); // size
        test_vm.set_storage(U256::ZERO, value);
        let value = B256::from([0xfa; 32]); // contents
        test_vm.set_storage(*storage.base(), value);

        storage.push(0xfa);

        let want: B256 = U256::from(33 * 2 + 1).into();
        assert_eq!(test_vm.get_storage(U256::ZERO), want);
        let base = *storage.base();
        let want = value;
        assert_eq!(test_vm.get_storage(base), want);
        let slot = base.saturating_add(U256::from(1));
        let want = B256::right_padding_from(&[0xfa]);
        assert_eq!(test_vm.get_storage(slot), want);
    }

    #[test]
    fn test_storage_bytes_push_growing() {
        let test_vm = TestVM::new();
        let mut storage = StorageBytes::from(&test_vm);

        let mut value = B256::from([0xfa; 32]);
        value[31] = 31 * 2;
        test_vm.set_storage(U256::ZERO, value);

        storage.push(0xfa);

        let want: B256 = U256::from(32 * 2 + 1).into();
        assert_eq!(test_vm.get_storage(U256::ZERO), want);
        let want = B256::from([0xfa; 32]);
        assert_eq!(test_vm.get_storage(*storage.base()), want);
    }

    #[test]
    fn test_storage_bytes_pop_small() {
        let test_vm = TestVM::new();
        let mut storage = StorageBytes::from(&test_vm);

        let mut data: Vec<u8> = (0..31).collect();
        data.push(31 * 2);
        test_vm.set_storage(U256::ZERO, B256::from_slice(&data));

        let mut want = B256::from_slice(&data);
        for i in (0..31).rev() {
            let popped = storage.pop();
            assert_eq!(popped, Some(i));
            want[i as usize] = 0x0;
            want[31] = i * 2;
            assert_eq!(test_vm.get_storage(U256::ZERO), want);
        }

        let value = storage.pop();
        assert!(value.is_none());
        assert_eq!(test_vm.get_storage(U256::ZERO), B256::ZERO);
    }

    #[test]
    fn test_storage_bytes_pop_big() {
        let test_vm = TestVM::new();
        let mut storage = StorageBytes::from(&test_vm);

        let value: B256 = U256::from(64 * 2 + 1).into();
        test_vm.set_storage(U256::ZERO, value);

        let data: Vec<u8> = (0..64).collect();
        let base = *storage.base();
        let after_base = base.saturating_add(U256::from(1));
        test_vm.set_storage(base, B256::from_slice(&data[0..32]));
        test_vm.set_storage(after_base, B256::from_slice(&data[32..64]));

        for i in (32..64).rev() {
            let popped = storage.pop();
            assert_eq!(popped, Some(i));
            let want_size: B256 = U256::from(i * 2 + 1).into();
            assert_eq!(test_vm.get_storage(U256::ZERO), want_size);
        }

        assert_eq!(test_vm.get_storage(base), B256::from_slice(&data[0..32]));
        assert_eq!(test_vm.get_storage(after_base), B256::ZERO);
    }

    #[test]
    fn test_storage_bytes_pop_shrinking() {
        let test_vm = TestVM::new();
        let mut storage = StorageBytes::from(&test_vm);

        let value: B256 = U256::from(32 * 2 + 1).into();
        test_vm.set_storage(U256::ZERO, value);

        let data: Vec<u8> = (0..32).collect();
        let base = *storage.base();
        test_vm.set_storage(base, B256::from_slice(&data[0..32]));

        let popped = storage.pop();
        assert_eq!(popped, Some(31));

        assert_eq!(test_vm.get_storage(base), B256::ZERO);
        let mut want = B256::right_padding_from(&data[0..31]);
        want[31] = 31 * 2;
        assert_eq!(test_vm.get_storage(U256::ZERO), want);
    }

    #[test]
    fn test_storage_bytes_get_small() {
        let test_vm = TestVM::new();
        let storage = StorageBytes::from(&test_vm);

        let mut value = (0..31).collect::<Vec<u8>>();
        value.push(31 * 2);
        test_vm.set_storage(U256::ZERO, B256::from_slice(&value));

        for i in 0..31 {
            let got = storage.get(i);
            assert_eq!(got, Some(i));
        }

        let got = storage.get(31);
        assert!(got.is_none());
    }

    #[test]
    fn test_storage_bytes_get_big() {
        let test_vm = TestVM::new();
        let storage = StorageBytes::from(&test_vm);

        let value: B256 = U256::from(64 * 2 + 1).into();
        test_vm.set_storage(U256::ZERO, value);

        let data: Vec<u8> = (0..64).collect();
        let base = *storage.base();
        test_vm.set_storage(base, B256::from_slice(&data[0..32]));
        test_vm.set_storage(
            base.saturating_add(U256::from(1)),
            B256::from_slice(&data[32..64]),
        );

        for i in 0..64 {
            let got = storage.get(i);
            assert_eq!(got, Some(i));
        }

        let got = storage.get(64);
        assert!(got.is_none());
    }

    #[test]
    fn test_storage_bytes_get_mut_small() {
        let test_vm = TestVM::new();
        let mut storage = StorageBytes::from(&test_vm);

        let mut value = B256::from(&[0xbe; 32]);
        value[31] = 31 * 2;
        test_vm.set_storage(U256::ZERO, value);

        let mut want = value;
        for i in 0..31 {
            let mut cell = storage.get_mut(i).unwrap();
            assert_eq!(cell.get(), alloy_primitives::fixed_bytes!("be"));
            cell.set(alloy_primitives::fixed_bytes!("af"));
            want[i as usize] = 0xaf;
            assert_eq!(test_vm.get_storage(U256::ZERO), want);
        }

        let got = storage.get_mut(31);
        assert!(got.is_none());
    }

    #[test]
    fn test_storage_bytes_get_mut_big() {
        let test_vm = TestVM::new();
        let mut storage = StorageBytes::from(&test_vm);

        let value: B256 = U256::from(32 * 2 + 1).into();
        test_vm.set_storage(U256::ZERO, value);
        let base = *storage.base();
        let value = B256::from(&[0xbe; 32]);
        test_vm.set_storage(base, value);

        let mut want = value;
        for i in 0..32 {
            let mut cell = storage.get_mut(i).unwrap();
            assert_eq!(cell.get(), alloy_primitives::fixed_bytes!("be"));
            cell.set(alloy_primitives::fixed_bytes!("af"));
            want[i as usize] = 0xaf;
            assert_eq!(test_vm.get_storage(base), want);
        }

        let got = storage.get_mut(32);
        assert!(got.is_none());
    }

    #[test]
    fn test_storage_bytes_get_bytes_small() {
        let test_vm = TestVM::new();
        let storage = StorageBytes::from(&test_vm);

        let mut value = (0..31).collect::<Vec<u8>>();
        value.push(31 * 2);
        test_vm.set_storage(U256::ZERO, B256::from_slice(&value));

        let got = storage.get_bytes();
        assert_eq!(&got, &value[0..31]);
    }

    #[test]
    fn test_storage_bytes_get_bytes_big() {
        let test_vm = TestVM::new();
        let storage = StorageBytes::from(&test_vm);

        let value: B256 = U256::from(64 * 2 + 1).into();
        test_vm.set_storage(U256::ZERO, value);

        let data: Vec<u8> = (0..64).collect();
        let base = *storage.base();
        let after_base = base.saturating_add(U256::from(1));
        test_vm.set_storage(base, B256::from_slice(&data[0..32]));
        test_vm.set_storage(after_base, B256::from_slice(&data[32..64]));

        let got = storage.get_bytes();
        assert_eq!(got, data);
    }

    #[test]
    fn test_storage_bytes_set_bytes_small() {
        let test_vm = TestVM::new();
        let mut storage = StorageBytes::from(&test_vm);

        let value: B256 = U256::from(64 * 2 + 1).into();
        test_vm.set_storage(U256::ZERO, value);

        let data: Vec<u8> = (0..64).collect();
        let base = *storage.base();
        let after_base = base.saturating_add(U256::from(1));
        test_vm.set_storage(base, B256::from_slice(&data[0..32]));
        test_vm.set_storage(after_base, B256::from_slice(&data[32..64]));

        let new_data: Vec<u8> = (100..116).collect();
        storage.set_bytes(&new_data);

        let mut want = B256::right_padding_from(&new_data);
        want[31] = new_data.len() as u8 * 2;
        assert_eq!(test_vm.get_storage(U256::ZERO), want);
        assert_eq!(test_vm.get_storage(base), B256::ZERO);
        assert_eq!(test_vm.get_storage(after_base), B256::ZERO);
    }

    #[test]
    fn test_storage_bytes_set_bytes_big() {
        let test_vm = TestVM::new();
        let mut storage = StorageBytes::from(&test_vm);

        let value: B256 = U256::from(64 * 2 + 1).into();
        test_vm.set_storage(U256::ZERO, value);

        let data: Vec<u8> = (0..64).collect();
        let base = *storage.base();
        let after_base = base.saturating_add(U256::from(1));
        test_vm.set_storage(base, B256::from_slice(&data[0..32]));
        test_vm.set_storage(after_base, B256::from_slice(&data[32..64]));

        let new_data: Vec<u8> = (100..132).collect();
        storage.set_bytes(&new_data);

        let want: B256 = U256::from(32 * 2 + 1).into();
        assert_eq!(test_vm.get_storage(U256::ZERO), want);
        assert_eq!(test_vm.get_storage(base), B256::from_slice(&new_data));
        assert_eq!(test_vm.get_storage(after_base), B256::ZERO);
    }

    #[test]
    fn test_storage_bytes_index_slot_small() {
        let test_vm = TestVM::new();
        let storage = StorageBytes::from(&test_vm);

        let cases = vec![(0, U256::ZERO, 0), (1, U256::ZERO, 1), (31, U256::ZERO, 31)];

        for (index, slot, offset) in cases {
            let root = BytesRoot::new(&storage);
            assert_eq!(root.index_slot(index), (slot, offset));
        }
    }

    #[test]
    fn test_storage_bytes_index_slot_big() {
        let test_vm = TestVM::new();
        let storage = StorageBytes::from(&test_vm);

        let value: B256 = U256::from(32 * 2 + 1).into();
        test_vm.set_storage(U256::ZERO, value);

        let base = *storage.base();
        let cases = vec![
            (0, base, 0),
            (1, base, 1),
            (31, base, 31),
            (32, base.saturating_add(U256::from(1)), 0),
            (33, base.saturating_add(U256::from(1)), 1),
            (63, base.saturating_add(U256::from(1)), 31),
        ];

        for (index, slot, offset) in cases {
            let root = BytesRoot::new(&storage);
            assert_eq!(root.index_slot(index), (slot, offset));
        }
    }

    #[test]
    fn test_storage_bytes_base() {
        let test_vm = TestVM::new();
        let storage = StorageBytes::from(&test_vm);
        let want: U256 = crypto::keccak([0; 32]).into();
        assert_eq!(*storage.base(), want);
    }

    #[test]
    fn test_storage_bytes_erase() {
        let test_vm = TestVM::new();
        let mut storage = StorageBytes::from(&test_vm);

        let size = 300;
        let value: B256 = U256::from(size * 2 + 1).into();
        test_vm.set_storage(U256::ZERO, value);

        let n_words = size / 32 + 1;
        let base = *storage.base();
        for i in 0..n_words {
            let word = B256::from([0xfa; 32]);
            let slot = base.saturating_add(U256::from(i));
            test_vm.set_storage(slot, word);
        }

        storage.erase();

        for i in 0..n_words {
            let slot = base.saturating_add(U256::from(i));
            assert_eq!(test_vm.get_storage(slot), B256::ZERO);
        }
        assert_eq!(test_vm.get_storage(U256::ZERO), B256::ZERO);
    }

    #[test]
    fn test_storage_bytes_extend() {
        let test_vm = TestVM::new();
        let mut storage = StorageBytes::from(&test_vm);

        let mut data: Vec<u8> = (0..15).collect();
        storage.extend(&data);
        let mut want = B256::right_padding_from(&data);
        want[31] = data.len() as u8 * 2;
        assert_eq!(test_vm.get_storage(U256::ZERO), want);

        data.extend(15..30);
        storage.extend(15..30);
        let mut want = B256::right_padding_from(&data);
        want[31] = data.len() as u8 * 2;
        assert_eq!(test_vm.get_storage(U256::ZERO), want);

        data.extend(30..45);
        storage.extend(30..45);
        let want: B256 = U256::from(data.len() as u8 * 2 + 1).into();
        assert_eq!(test_vm.get_storage(U256::ZERO), want);
        let base = *storage.base();
        assert_eq!(test_vm.get_storage(base), B256::from_slice(&data[0..32]));
        let after_base = base.saturating_add(U256::from(1));
        assert_eq!(
            test_vm.get_storage(after_base),
            B256::right_padding_from(&data[32..45])
        );
    }
}
