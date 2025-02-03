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
        if (len < 32) && (old >= 32) {
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

        let cases = vec![
            (0, B256::ZERO),
            (5, U256::from(5 * 2).into()),
            (500, U256::from(500 * 2 + 1).into()),
        ];

        for (len, want) in cases {
            unsafe {
                storage.write_len(len);
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

        let mut want = value.clone();
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
            assert_eq!(storage.index_slot(index), (slot, offset));
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
            assert_eq!(storage.index_slot(index), (slot, offset));
        }
    }

    #[test]
    fn test_storage_bytes_base() {
        let test_vm = TestVM::new();
        let storage = StorageBytes::from(&test_vm);
        let want: U256 = crypto::keccak(&[0; 32]).into();
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
