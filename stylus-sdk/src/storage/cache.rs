// Copyright 2023, Offchain Labs, Inc.
// For license information, see https://github.com/OffchainLabs/nitro/blob/master/LICENSE

use crate::{load_bytes32, store_bytes32};
use alloy_primitives::{FixedBytes, Signed, Uint, B256, U256};
use derivative::Derivative;
use fnv::FnvHashMap as HashMap;
use lazy_static::lazy_static;
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr,
    sync::Mutex,
};

/// Global cache managing persistent storage operations.
pub struct StorageCache(HashMap<U256, StorageWord>);

/// Represents the EVM word at a given key.
pub struct StorageWord {
    /// The current value of the slot.
    value: B256,
    /// The value in the EVM state trie, if known.
    known: Option<B256>,
}

impl StorageWord {
    /// Creates a new slot from a known value in the EVM state trie.
    fn new_known(known: B256) -> Self {
        Self {
            value: known,
            known: Some(known),
        }
    }

    /// Creates a new slot without knowing the underlying value in the EVM state trie.
    fn new_unknown(value: B256) -> Self {
        Self { value, known: None }
    }

    /// Whether a slot should be written to disk.
    fn dirty(&self) -> bool {
        Some(self.value) != self.known
    }
}

lazy_static! {
    /// Global cache managing persistent storage operations.
    static ref CACHE: Mutex<StorageCache> = Mutex::new(StorageCache(HashMap::default()));
}

macro_rules! cache {
    () => {
        CACHE.lock().unwrap().0
    };
}

impl StorageCache {
    /// Retrieves `N ≤ 32` bytes from persistent storage, performing [`SLOAD`]'s only as needed.
    /// The bytes are read from slot `key`, starting `offset` bytes from the left.
    /// Note that the bytes must exist within a single, 32-byte EVM word.
    ///
    /// # Safety
    ///
    /// UB if the read would cross a word boundary.
    /// May become safe when Rust stabilizes [`generic_const_exprs`].
    ///
    /// [`SLOAD`]: https://www.evm.codes/#54
    /// [`generic_const_exprs`]: https://github.com/rust-lang/rust/issues/76560
    pub unsafe fn get<const N: usize>(key: U256, offset: usize) -> FixedBytes<N> {
        debug_assert!(N + offset <= 32);
        let word = Self::get_word(key);
        let value = &word[offset..][..N];
        FixedBytes::from_slice(value)
    }

    /// Retrieves a [`Uint`] from persistent storage, performing [`SLOAD`]'s only as needed.
    /// The integer's bytes are read from slot `key`, starting `offset` bytes from the left.
    /// Note that the bytes must exist within a single, 32-byte EVM word.
    ///
    /// # Safety
    ///
    /// UB if the read would cross a word boundary.
    /// May become safe when Rust stabilizes [`generic_const_exprs`].
    ///
    /// [`SLOAD`]: https://www.evm.codes/#54
    /// [`generic_const_exprs`]: https://github.com/rust-lang/rust/issues/76560
    pub unsafe fn get_uint<const B: usize, const L: usize>(key: U256, offset: usize) -> Uint<B, L> {
        debug_assert!(B / 8 + offset <= 32);
        let word = Self::get_word(key);
        let value = &word[offset..][..B / 8];
        Uint::try_from_be_slice(value).unwrap()
    }

    /// Retrieves a [`u8`] from persistent storage, performing [`SLOAD`]'s only as needed.
    /// The byte is read from slot `key`, starting `offset` bytes from the left.
    ///
    /// # Safety
    ///
    /// UB if the read is out of bounds.
    /// May become safe when Rust stabilizes [`generic_const_exprs`].
    ///
    /// [`SLOAD`]: https://www.evm.codes/#54
    /// [`generic_const_exprs`]: https://github.com/rust-lang/rust/issues/76560
    pub unsafe fn get_byte(key: U256, offset: usize) -> u8 {
        debug_assert!(offset <= 32);
        let word = Self::get::<1>(key, offset);
        word[0]
    }

    /// Retrieves a [`Signed`] from persistent storage, performing [`SLOAD`]'s only as needed.
    /// The integer's bytes are read from slot `key`, starting `offset` bytes from the left.
    /// Note that the bytes must exist within a single, 32-byte EVM word.
    ///
    /// # Safety
    ///
    /// UB if the read would cross a word boundary.
    /// May become safe when Rust stabilizes [`generic_const_exprs`].
    ///
    /// [`SLOAD`]: https://www.evm.codes/#54
    /// [`generic_const_exprs`]: https://github.com/rust-lang/rust/issues/76560
    pub unsafe fn get_signed<const B: usize, const L: usize>(
        key: U256,
        offset: usize,
    ) -> Signed<B, L> {
        Signed::from_raw(Self::get_uint(key, offset))
    }

    /// Retrieves a 32-byte EVM word from persistent storage, performing [`SLOAD`]'s only as needed.
    ///
    /// [`SLOAD`]: https://www.evm.codes/#54
    pub fn get_word(key: U256) -> B256 {
        cache!()
            .entry(key)
            .or_insert_with(|| StorageWord::new_known(load_bytes32(key)))
            .value
    }

    /// Writes `N ≤ 32` bytes to persistent storage, performing [`SSTORE`]'s only as needed.
    /// The bytes are written to slot `key`, starting `offset` bytes from the left.
    /// Note that the bytes must be written to a single, 32-byte EVM word.
    ///
    /// # Safety
    ///
    /// UB if the write would cross a word boundary.
    /// Aliases if called during the lifetime an overlapping accessor.
    ///
    /// [`SSTORE`]: https://www.evm.codes/#55
    pub unsafe fn set<const N: usize>(key: U256, offset: usize, value: FixedBytes<N>) {
        debug_assert!(N + offset <= 32);

        if N == 32 {
            return Self::set_word(key, FixedBytes::from_slice(value.as_slice()));
        }

        let cache = &mut cache!();
        let word = cache
            .entry(key)
            .or_insert_with(|| StorageWord::new_known(load_bytes32(key)));

        let dest = word.value[offset..].as_mut_ptr();
        ptr::copy(value.as_ptr(), dest, N)
    }

    /// Writes a [`Uint`] to persistent storage, performing [`SSTORE`]'s only as needed.
    /// The integer's bytes are written to slot `key`, starting `offset` bytes from the left.
    /// Note that the bytes must be written to a single, 32-byte EVM word.
    ///
    /// # Safety
    ///
    /// UB if the write would cross a word boundary.
    /// Aliases if called during the lifetime an overlapping accessor.
    ///
    /// [`SSTORE`]: https://www.evm.codes/#55
    pub unsafe fn set_uint<const B: usize, const L: usize>(
        key: U256,
        offset: usize,
        value: Uint<B, L>,
    ) {
        debug_assert!(B / 8 + offset <= 32);

        if B == 256 {
            return Self::set_word(key, FixedBytes::from_slice(&value.to_be_bytes::<32>()));
        }

        let cache = &mut cache!();
        let word = cache
            .entry(key)
            .or_insert_with(|| StorageWord::new_known(load_bytes32(key)));

        let value = value.to_be_bytes_vec();
        let dest = word.value[offset..].as_mut_ptr();
        ptr::copy(value.as_ptr(), dest, B / 8)
    }

    /// Writes a [`Signed`] to persistent storage, performing [`SSTORE`]'s only as needed.
    /// The bytes are written to slot `key`, starting `offset` bytes from the left.
    /// Note that the bytes must be written to a single, 32-byte EVM word.
    ///
    /// # Safety
    ///
    /// UB if the write would cross a word boundary.
    /// Aliases if called during the lifetime an overlapping accessor.
    ///
    /// [`SSTORE`]: https://www.evm.codes/#55
    pub unsafe fn set_signed<const B: usize, const L: usize>(
        key: U256,
        offset: usize,
        value: Signed<B, L>,
    ) {
        Self::set_uint(key, offset, value.into_raw())
    }

    /// Writes a [`u8`] to persistent storage, performing [`SSTORE`]'s only as needed.
    /// The byte is written to slot `key`, starting `offset` bytes from the left.
    ///
    /// # Safety
    ///
    /// UB if the write is out of bounds.
    /// Aliases if called during the lifetime an overlapping accessor.
    ///
    /// [`SSTORE`]: https://www.evm.codes/#55
    pub unsafe fn set_byte(key: U256, offset: usize, value: u8) {
        let fixed = FixedBytes::from_slice(&[value]);
        StorageCache::set::<1>(key, offset, fixed)
    }

    /// Stores a 32-byte EVM word to persistent storage, performing [`SSTORE`]'s only as needed.
    ///
    /// # Safety
    ///
    /// Aliases if called during the lifetime an overlapping accessor.
    ///
    /// [`SSTORE`]: https://www.evm.codes/#55
    pub unsafe fn set_word(key: U256, value: B256) {
        cache!().insert(key, StorageWord::new_unknown(value));
    }

    /// Write all cached values to persistent storage.
    /// Note: this operation retains [`SLOAD`] information for optimization purposes.
    /// If reentrancy is possible, use [`StorageCache::clear`].
    ///
    /// [`SLOAD`]: https://www.evm.codes/#54
    pub fn flush() {
        for (key, entry) in &mut cache!() {
            if entry.dirty() {
                store_bytes32(*key, entry.value);
            }
        }
    }

    /// Flush and clear the storage cache.
    pub fn clear() {
        StorageCache::flush();
        cache!().clear();
    }
}

/// Accessor trait that lets a type be used in persistent storage.
/// Users can implement this trait to add novel data structures to their contract definitions.
/// The Stylus SDK by default provides only solidity types, which are represented [`the same way`].
///
/// [`the same way`]: https://docs.soliditylang.org/en/v0.8.15/internals/layout_in_storage.html
pub trait StorageType: Sized {
    /// For primative types, this is the type being stored.
    /// For collections, this is the [`StorageType`] being collected.
    type Wraps<'a>: 'a
    where
        Self: 'a;

    /// Mutable accessor to the type being stored.
    type WrapsMut<'a>: 'a
    where
        Self: 'a;

    /// The number of bytes in a slot needed to represent the type. Must not exceed 32.
    /// For types larger than 32 bytes that are stored inline with a struct's fields,
    /// set this to 32 and return the full size in [`StorageType::new`].
    ///
    /// For implementing collections, see how Solidity slots are assigned for [`Arrays and Maps`] and their
    /// Stylus equivalents [`StorageVec`] and [`StorageMap`].
    /// For multi-word, but still-fixed-size types, see the implementations for structs and [`StorageArray`].
    ///
    /// [`Arrays and Maps`]: https://docs.soliditylang.org/en/v0.8.15/internals/layout_in_storage.html#mappings-and-dynamic-arrays
    const SLOT_BYTES: usize = 32;

    /// The number of words this type must fill. For primitives this is always 0.
    /// For complex types requiring more than one inline word, set this to the total size.
    const REQUIRED_SLOTS: usize = 0;

    /// Where in persistent storage the type should live. Although useful for framework designers
    /// creating new storage types, most user programs shouldn't call this.
    /// Note: implementations will have to be `const` once [`generic_const_exprs`] stabilizes.
    ///
    /// # Safety
    ///
    /// Aliases storage if two calls to the same slot and offset occur within the same lifetime.
    ///
    /// [`generic_const_exprs`]: https://github.com/rust-lang/rust/issues/76560
    unsafe fn new(slot: U256, offset: u8) -> Self;

    /// Load the wrapped type, consuming the accessor.
    /// Note: most types have a `get` and/or `getter`, which don't consume `Self`.
    fn load<'s>(self) -> Self::Wraps<'s>
    where
        Self: 's;

    /// Load the wrapped mutable type, consuming the accessor.
    /// Note: most types have a `set` and/or `setter`, which don't consume `Self`.
    fn load_mut<'s>(self) -> Self::WrapsMut<'s>
    where
        Self: 's;
}

/// Trait for accessors that can be used to completely erase their underlying value.
/// Note that some collections, like [`StorageMap`], don't implement this trait.
pub trait EraseStorageType: StorageType {
    /// Erase the value from persistent storage.
    fn erase(&mut self);
}

/// Trait for simple accessors that store no more than their wrapped value.
/// The type's representation must be entirely inline, or storage leaks become possible.
/// Note: it is a logic error if erasure does anything more than writing the zero-value.
pub trait SimpleStorageType<'a>: StorageType + EraseStorageType + Into<Self::Wraps<'a>>
where
    Self: 'a,
{
    /// Write the value to persistent storage.
    fn set_by_wrapped(&mut self, value: Self::Wraps<'a>);
}

/// Binds a storage accessor to a lifetime to prevent aliasing.
/// Because this type doesn't implement `DerefMut`, mutable methods on the accessor aren't available.
/// For a mutable accessor, see [`StorageGuardMut`].
#[derive(Derivative)]
#[derivative(Debug = "transparent")]
pub struct StorageGuard<'a, T: 'a> {
    inner: T,
    #[derivative(Debug = "ignore")]
    marker: PhantomData<&'a T>,
}

impl<'a, T: 'a> StorageGuard<'a, T> {
    /// Creates a new storage guard around an arbitrary type.
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }

    /// Get the underlying `T` directly, bypassing the borrow checker.
    ///
    /// # Safety
    ///
    /// Enables storage aliasing.
    pub unsafe fn into_raw(self) -> T {
        self.inner
    }
}

impl<'a, T: 'a> Deref for StorageGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Binds a storage accessor to a lifetime to prevent aliasing.
pub struct StorageGuardMut<'a, T: 'a> {
    inner: T,
    marker: PhantomData<&'a T>,
}

impl<'a, T: 'a> StorageGuardMut<'a, T> {
    /// Creates a new storage guard around an arbitrary type.
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }

    /// Get the underlying `T` directly, bypassing the borrow checker.
    ///
    /// # Safety
    ///
    /// Enables storage aliasing.
    pub unsafe fn into_raw(self) -> T {
        self.inner
    }
}

impl<'a, T: 'a> Deref for StorageGuardMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T: 'a> DerefMut for StorageGuardMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
