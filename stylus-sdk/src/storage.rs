// Copyright 2023, Offchain Labs, Inc.
// For license information, see https://github.com/OffchainLabs/nitro/blob/master/LICENSE

use crate::{crypto, load_bytes32, store_bytes32};
use alloy_primitives::{Address, BlockHash, BlockNumber, FixedBytes, Signed, Uint, B256, U256};
use fnv::FnvHashMap as HashMap;
use lazy_static::lazy_static;
use std::{
    cell::OnceCell,
    marker::PhantomData,
    mem::transmute,
    ops::{Deref, DerefMut},
    ptr,
    slice::SliceIndex,
    sync::Mutex,
};

/// Global cache managing permanent storage operations
pub struct StorageCache(HashMap<B256, StorageWord>);

/// Represents the EVM word at a given key
pub struct StorageWord {
    /// The current value of the slot
    value: B256,
    /// The value in the EVM state trie, if known
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

    /// Whether a slot should be written to disk
    fn dirty(&self) -> bool {
        Some(self.value) != self.known
    }
}

lazy_static! {
    /// Global cache managing permanent storage operations
    static ref CACHE: Mutex<StorageCache> = Mutex::new(StorageCache(HashMap::default()));
}

macro_rules! cache {
    () => {
        CACHE.lock().unwrap().0
    };
}

impl StorageCache {
    /// Retrieves `N ≤ 32` bytes from permanent storage, performing [`SLOAD`]'s only as needed.
    /// The bytes are read from slot `key`, starting `offset` bytes from the right.
    /// Note that the bytes must exist within a single, 32-byte EVM word.
    ///
    /// # Safety
    ///
    /// UB if the read would cross a word boundary.
    /// May become safe when Rust stabilizes [`generic_const_exprs`].
    ///
    /// [`SLOAD`]: https://www.evm.codes/#54
    /// [`generic_const_exprs`]: https://github.com/rust-lang/rust/issues/76560
    pub unsafe fn get<const N: usize>(key: B256, offset: usize) -> FixedBytes<N> {
        debug_assert!(N + offset <= 32);
        let word = Self::get_word(key);
        let (_, value) = word.split_at(offset);
        FixedBytes::from_slice(value)
    }

    /// Retrieves a [`Uint`] from permanent storage, performing [`SLOAD`]'s only as needed.
    /// The integer's bytes are read from slot `key`, starting `offset` bytes from the right.
    /// Note that the bytes must exist within a single, 32-byte EVM word.
    ///
    /// # Safety
    ///
    /// UB if the read would cross a word boundary.
    /// May become safe when Rust stabilizes [`generic_const_exprs`].
    ///
    /// [`SLOAD`]: https://www.evm.codes/#54
    /// [`generic_const_exprs`]: https://github.com/rust-lang/rust/issues/76560
    pub unsafe fn get_uint<const B: usize, const L: usize>(key: B256, offset: usize) -> Uint<B, L> {
        debug_assert!(B / 8 + offset <= 32);
        let word = Self::get_word(key);
        let (_, value) = word.split_at(offset);
        Uint::try_from_be_slice(value).unwrap()
    }

    /// Retrieves a [`Signed`] from permanent storage, performing [`SLOAD`]'s only as needed.
    /// The integer's bytes are read from slot `key`, starting `offset` bytes from the right.
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
        key: B256,
        offset: usize,
    ) -> Signed<B, L> {
        Signed::from_raw(Self::get_uint(key, offset))
    }

    /// Retrieves a 32-byte EVM word from permanent storage, performing [`SLOAD`]'s only as needed.
    ///
    /// [`SLOAD`]: https://www.evm.codes/#54
    pub fn get_word(key: B256) -> B256 {
        cache!()
            .entry(key)
            .or_insert_with(|| StorageWord::new_known(load_bytes32(key)))
            .value
    }

    /// Writes `N ≤ 32` bytes to permanent storage, performing [`SSTORE`]'s only as needed.
    /// The bytes are written to slot `key`, starting `offset` bytes from the right.
    /// Note that the bytes must be written to a single, 32-byte EVM word.
    ///
    /// # Safety
    ///
    /// UB if the write would cross a word boundary.
    /// May become safe when Rust stabilizes [`generic_const_exprs`].
    ///
    /// [`SSTORE`]: https://www.evm.codes/#55
    /// [`generic_const_exprs`]: https://github.com/rust-lang/rust/issues/76560
    pub unsafe fn set<const N: usize>(key: B256, offset: usize, value: FixedBytes<N>) {
        debug_assert!(N + offset <= 32);

        if N == 32 {
            return Self::set_word(key, FixedBytes::from_slice(value.as_slice()));
        }

        let cache = &mut cache!();
        let word = cache
            .entry(key)
            .or_insert_with(|| StorageWord::new_known(load_bytes32(key)));

        ptr::copy(value.as_ptr(), word.value[32 - N..].as_mut_ptr(), N)
    }

    /// Writes a [`Uint`] to permanent storage, performing [`SSTORE`]'s only as needed.
    /// The integer's bytes are written to slot `key`, starting `offset` bytes from the right.
    /// Note that the bytes must be written to a single, 32-byte EVM word.
    ///
    /// # Safety
    ///
    /// UB if the write would cross a word boundary.
    /// May become safe when Rust stabilizes [`generic_const_exprs`].
    ///
    /// [`SSTORE`]: https://www.evm.codes/#55
    /// [`generic_const_exprs`]: https://github.com/rust-lang/rust/issues/76560
    pub unsafe fn set_uint<const B: usize, const L: usize>(
        key: B256,
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

        let value = value.as_le_bytes();
        ptr::copy(value.as_ptr(), word.value[32 - B / 8..].as_mut_ptr(), B / 8)
    }

    /// Writes a [`Signed`] to permanent storage, performing [`SSTORE`]'s only as needed.
    /// The bytes are written to slot `key`, starting `offset` bytes from the right.
    /// Note that the bytes must be written to a single, 32-byte EVM word.
    ///
    /// # Safety
    ///
    /// UB if the write would cross a word boundary.
    /// May become safe when Rust stabilizes [`generic_const_exprs`].
    ///
    /// [`SSTORE`]: https://www.evm.codes/#55
    /// [`generic_const_exprs`]: https://github.com/rust-lang/rust/issues/76560
    pub unsafe fn set_signed<const B: usize, const L: usize>(
        key: B256,
        offset: usize,
        value: Signed<B, L>,
    ) {
        Self::set_uint(key, offset, value.into_raw())
    }

    /// Stores a 32-byte EVM word to permanent storage, performing [`SSTORE`]'s only as needed.
    ///
    /// [`SSTORE`]: https://www.evm.codes/#55
    pub fn set_word(key: B256, value: B256) {
        cache!().insert(key, StorageWord::new_unknown(value));
    }

    /// Write all cached values to permanent storage.
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

// TODO: use const generics once stable to elide runtime keccaks
pub trait StorageType {
    const SIZE: u8 = 32;

    fn new(slot: U256, offset: u8) -> Self;
}

/// Binds a storage accessor to a lifetime to prevent aliasing.
/// Because this type doesn't implement `DerefMut`, mutable methods on the accessor aren't available.
/// For a mutable accessor, see [`StorageGuardMut`].
pub struct StorageGuard<'a, T: 'a> {
    inner: T,
    marker: PhantomData<&'a T>,
}

impl<'a, T: 'a> StorageGuard<'a, T> {
    fn new(inner: T) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
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
    fn new(inner: T) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
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

macro_rules! alias_ints {
    ($($name:ident, $signed_name:ident, $bits:expr, $limbs:expr;)*) => {
        $(
            #[doc = concat!("Accessor for a storage-backed [`U", stringify!($bits), "`]")]
            pub type $name = StorageUint<$bits, $limbs>;

            #[doc = concat!("Accessor for a storage-backed [`I", stringify!($bits), "`]")]
            pub type $signed_name = StorageSigned<$bits, $limbs>;
        )*
    };
}

macro_rules! alias_bytes {
    ($($name:ident, $bytes:expr;)*) => {
        $(
            #[doc = concat!("Accessor for a storage-backed [`B", stringify!($bytes), "`]")]
            pub type $name = StorageFixedBytes<$bytes>;
        )*
    };
}

alias_ints! {
    StorageU0, StorageI0, 0, 0;
    StorageU1, StorageI1, 1, 1;
    StorageU8, StorageI8, 8, 1;
    StorageU16, StorageI16, 16, 1;
    StorageU32, StorageI32, 32, 1;
    StorageU64, StorageI64, 64, 1;
    StorageU128, StorageI128, 128, 2;
    StorageU160, StorageI160, 160, 3;
    StorageU192, StorageI192, 192, 3;
    StorageU256, StorageI256, 256, 4;
}

alias_bytes! {
    StorageB0, 0;
    StorageB8, 1;
    StorageB16, 2;
    StorageB32, 4;
    StorageB64, 8;
    StorageB96, 12;
    StorageB128, 16;
    StorageB160, 20;
    StorageB192, 24;
    StorageB224, 28;
    StorageB256, 32;
}

pub struct StorageUint<const B: usize, const L: usize> {
    slot: U256,
    offset: u8,
}

impl<const B: usize, const L: usize> StorageUint<B, L> {
    pub fn get(&self) -> Uint<B, L> {
        unsafe { StorageCache::get_uint(self.slot.into(), self.offset.into()) }
    }
}

impl<const B: usize, const L: usize> StorageType for StorageUint<B, L> {
    const SIZE: u8 = (B / 8) as u8;

    fn new(slot: U256, offset: u8) -> Self {
        debug_assert!(B <= 256);
        Self { slot, offset }
    }
}

pub struct StorageSigned<const B: usize, const L: usize> {
    slot: U256,
    offset: u8,
}

impl<const B: usize, const L: usize> StorageSigned<B, L> {
    pub fn get(&self) -> Signed<B, L> {
        unsafe { StorageCache::get_signed(self.slot.into(), self.offset.into()) }
    }
}

impl<const B: usize, const L: usize> StorageType for StorageSigned<B, L> {
    const SIZE: u8 = (B / 8) as u8;

    fn new(slot: U256, offset: u8) -> Self {
        Self { slot, offset }
    }
}

pub struct StorageFixedBytes<const N: usize> {
    slot: U256,
    offset: u8,
}

impl<const N: usize> StorageFixedBytes<N> {
    pub fn get(&self) -> FixedBytes<N> {
        unsafe { StorageCache::get(self.slot.into(), self.offset.into()) }
    }
}

impl<const N: usize> StorageType for StorageFixedBytes<N> {
    const SIZE: u8 = N as u8;

    fn new(slot: U256, offset: u8) -> Self {
        Self { slot, offset }
    }
}

/// Accessor for a storage-backed [`Address`]
pub struct StorageAddress {
    slot: U256,
    offset: u8,
}

impl StorageAddress {
    pub fn get(&self) -> Address {
        let data = unsafe { StorageCache::get::<20>(self.slot.into(), self.offset.into()) };
        Address::from(data)
    }
}

impl StorageType for StorageAddress {
    const SIZE: u8 = 20;

    fn new(slot: U256, offset: u8) -> Self {
        Self { slot, offset }
    }
}

/// Accessor for a storage-backed [`BlockNumber`]
pub struct StorageBlockNumber {
    slot: U256,
    offset: u8,
}

impl StorageBlockNumber {
    pub fn get(&self) -> BlockNumber {
        let data = unsafe { StorageCache::get::<8>(self.slot.into(), self.offset.into()) };
        unsafe { transmute(data) }
    }
}

impl StorageType for StorageBlockNumber {
    const SIZE: u8 = 8;

    fn new(slot: U256, offset: u8) -> Self {
        Self { slot, offset }
    }
}

/// Accessor for a storage-backed [`BlockHash`]
pub struct StorageBlockHash {
    slot: U256,
}

impl StorageBlockHash {
    pub fn get(&self) -> BlockHash {
        StorageCache::get_word(self.slot.into())
    }
}

impl StorageType for StorageBlockHash {
    fn new(slot: U256, _offset: u8) -> Self {
        Self { slot }
    }
}

/// Accessor for a storage-backed vector
pub struct StorageVec<S: StorageType> {
    slot: U256,
    base: OnceCell<U256>,
    marker: PhantomData<S>,
}

impl<S: StorageType> StorageType for StorageVec<S> {
    fn new(slot: U256, offset: u8) -> Self {
        debug_assert!(offset == 0);
        Self {
            slot,
            base: OnceCell::new(),
            marker: PhantomData,
        }
    }
}

impl<S: StorageType> StorageVec<S> {
    /// Returns `true` if the collection contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    /// Gets the number of elements stored.
    pub fn len(&self) -> usize {
        let word: U256 = StorageCache::get_word(self.slot.into()).into();
        word.try_into().unwrap()
    }

    /// Gets an accessor to the element at a given index, if it exists.
    /// Note: the accessor is protected by a [`StoreageGuard`], which restricts
    /// its lifetime to that of `&self`.
    pub fn get<I>(&self, index: I) -> Option<StorageGuard<S>>
    where
        I: SliceIndex<[S]> + TryInto<usize>,
    {
        let accessor = unsafe { self.get_raw(index)? };
        Some(StorageGuard::new(accessor))
    }

    /// Gets a mutable accessor to the element at a given index, if it exists.
    /// Note: the accessor is protected by a [`StoreageGuardMut`], which restricts
    /// its lifetime to that of `&mut self`.
    pub fn get_mut<I>(&mut self, index: I) -> Option<StorageGuardMut<S>>
    where
        I: SliceIndex<[S]> + TryInto<usize>,
    {
        let accessor = unsafe { self.get_raw(index)? };
        Some(StorageGuardMut::new(accessor))
    }

    /// Gets the underlying accessor to the element at a given index, if it exists.
    ///
    /// # Safety
    ///
    /// Because the accessor is unconstrained by a storage guard, storage aliasing is possible
    /// if used incorrectly. Two or more mutable references to the same `S` are possible, as are
    /// read-after-write scenarios.
    pub unsafe fn get_raw<I>(&self, index: I) -> Option<S>
    where
        I: SliceIndex<[S]> + TryInto<usize>,
    {
        let index = index.try_into().ok()?;
        let width = S::SIZE as usize;

        if index > self.len() {
            return None;
        }

        let density = 32 / width;
        let offset = self.base() + U256::from(width * index / density);
        Some(S::new(offset, (index % density) as u8))
    }

    pub fn push(&mut self, _item: S) {
        let _index = self.len();
        todo!()
    }

    pub fn pop(&mut self) -> Option<S> {
        let _index = match self.len() {
            0 => return None,
            x => x - 1,
        };
        todo!()
    }

    /// Determines where in storage indices start. Could be made const in the future.
    fn base(&self) -> &U256 {
        self.base
            .get_or_init(|| crypto::keccak(self.slot.to_be_bytes::<32>()).into())
    }
}
