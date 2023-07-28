// Copyright 2023, Offchain Labs, Inc.
// For license information, see https://github.com/OffchainLabs/nitro/blob/master/LICENSE

use alloy_primitives::{Address, BlockHash, BlockNumber, FixedBytes, Signed, Uint, U256};
use std::{cell::OnceCell, mem::transmute, ops::Deref};

pub use bytes::StorageBytes;
pub use cache::{SizedStorageType, StorageCache, StorageGuard, StorageGuardMut, StorageType};
pub use map::StorageMap;
pub use vec::StorageVec;

pub mod bytes;
pub mod cache;
pub mod map;
pub mod vec;

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

/// Accessor for a storage-backed [`Uint`].
#[derive(Debug)]
pub struct StorageUint<const B: usize, const L: usize> {
    slot: U256,
    offset: u8,
    cached: OnceCell<Uint<B, L>>,
}

impl<const B: usize, const L: usize> StorageUint<B, L> {
    /// Gets the underlying [`Uint`] in persistent storage.
    pub fn get(&self) -> Uint<B, L> {
        **self
    }

    /// Sets the underlying [`Uint`] in persistent storage.
    pub fn set(&mut self, value: Uint<B, L>) {
        unsafe { StorageCache::set_uint(self.slot, self.offset.into(), value) };
    }
}

impl<const B: usize, const L: usize> StorageType for StorageUint<B, L> {
    const SIZE: u8 = (B / 8) as u8;

    fn new(slot: U256, offset: u8) -> Self {
        debug_assert!(B <= 256);
        Self {
            slot,
            offset,
            cached: OnceCell::new(),
        }
    }
}

impl<const B: usize, const L: usize> SizedStorageType for StorageUint<B, L> {
    type Value = Uint<B, L>;

    fn set_exact(&mut self, value: Self::Value) {
        self.set(value);
    }

    fn erase(&mut self) {
        self.set(Self::Value::ZERO);
    }
}

impl<const B: usize, const L: usize> Deref for StorageUint<B, L> {
    type Target = Uint<B, L>;

    fn deref(&self) -> &Self::Target {
        self.cached
            .get_or_init(|| unsafe { StorageCache::get_uint(self.slot, self.offset.into()) })
    }
}

impl<const B: usize, const L: usize> From<StorageUint<B, L>> for Uint<B, L> {
    fn from(value: StorageUint<B, L>) -> Self {
        *value
    }
}

/// Accessor for a storage-backed [`Signed`].
#[derive(Debug)]
pub struct StorageSigned<const B: usize, const L: usize> {
    slot: U256,
    offset: u8,
    cached: OnceCell<Signed<B, L>>,
}

impl<const B: usize, const L: usize> StorageSigned<B, L> {
    /// Gets the underlying [`Signed`] in persistent storage.
    pub fn get(&self) -> Signed<B, L> {
        **self
    }

    /// Gets the underlying [`Signed`] in persistent storage.
    pub fn set(&mut self, value: Signed<B, L>) {
        unsafe { StorageCache::set_signed(self.slot, self.offset.into(), value) };
    }
}

impl<const B: usize, const L: usize> StorageType for StorageSigned<B, L> {
    const SIZE: u8 = (B / 8) as u8;

    fn new(slot: U256, offset: u8) -> Self {
        Self {
            slot,
            offset,
            cached: OnceCell::new(),
        }
    }
}

impl<const B: usize, const L: usize> SizedStorageType for StorageSigned<B, L> {
    type Value = Signed<B, L>;

    fn set_exact(&mut self, value: Self::Value) {
        self.set(value);
    }

    fn erase(&mut self) {
        self.set(Self::Value::ZERO)
    }
}

impl<const B: usize, const L: usize> Deref for StorageSigned<B, L> {
    type Target = Signed<B, L>;

    fn deref(&self) -> &Self::Target {
        self.cached
            .get_or_init(|| unsafe { StorageCache::get_signed(self.slot, self.offset.into()) })
    }
}

impl<const B: usize, const L: usize> From<StorageSigned<B, L>> for Signed<B, L> {
    fn from(value: StorageSigned<B, L>) -> Self {
        *value
    }
}

/// Accessor for a storage-backed [`FixedBytes`].
#[derive(Debug)]
pub struct StorageFixedBytes<const N: usize> {
    slot: U256,
    offset: u8,
    cached: OnceCell<FixedBytes<N>>,
}

impl<const N: usize> StorageFixedBytes<N> {
    /// Gets the underlying [`FixedBytes`] in persistent storage.
    pub fn get(&self) -> FixedBytes<N> {
        **self
    }

    /// Gets the underlying [`FixedBytes`] in persistent storage.
    pub fn set(&mut self, value: FixedBytes<N>) {
        unsafe { StorageCache::set(self.slot, self.offset.into(), value) }
    }
}

impl<const N: usize> StorageType for StorageFixedBytes<N> {
    const SIZE: u8 = N as u8;

    fn new(slot: U256, offset: u8) -> Self {
        Self {
            slot,
            offset,
            cached: OnceCell::new(),
        }
    }
}

impl<const N: usize> SizedStorageType for StorageFixedBytes<N> {
    type Value = FixedBytes<N>;

    fn set_exact(&mut self, value: Self::Value) {
        self.set(value);
    }

    fn erase(&mut self) {
        self.set(Self::Value::ZERO)
    }
}

impl<const N: usize> Deref for StorageFixedBytes<N> {
    type Target = FixedBytes<N>;

    fn deref(&self) -> &Self::Target {
        self.cached
            .get_or_init(|| unsafe { StorageCache::get(self.slot, self.offset.into()) })
    }
}

impl<const N: usize> From<StorageFixedBytes<N>> for FixedBytes<N> {
    fn from(value: StorageFixedBytes<N>) -> Self {
        *value
    }
}

/// Accessor for a storage-backed [`bool`].
#[derive(Debug)]
pub struct StorageBool {
    slot: U256,
    offset: u8,
    cached: OnceCell<bool>,
}

impl StorageBool {
    /// Gets the underlying [`bool`] in persistent storage.
    pub fn get(&self) -> bool {
        **self
    }

    /// Gets the underlying [`bool`] in persistent storage.
    pub fn set(&mut self, value: bool) {
        self.cached.take();
        _ = self.cached.set(value);

        let value = value.then_some(1).unwrap_or_default();
        let fixed = FixedBytes::from_slice(&[value]);
        unsafe { StorageCache::set::<1>(self.slot, self.offset.into(), fixed) }
    }
}

impl StorageType for StorageBool {
    const SIZE: u8 = 1;

    fn new(slot: U256, offset: u8) -> Self {
        Self {
            slot,
            offset,
            cached: OnceCell::new(),
        }
    }
}

impl SizedStorageType for StorageBool {
    type Value = bool;

    fn set_exact(&mut self, value: Self::Value) {
        self.set(value);
    }

    fn erase(&mut self) {
        self.set(false);
    }
}

impl Deref for StorageBool {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        self.cached.get_or_init(|| unsafe {
            let data = StorageCache::get::<1>(self.slot, self.offset.into());
            data.as_slice()[0] != 0
        })
    }
}

impl From<StorageBool> for bool {
    fn from(value: StorageBool) -> Self {
        *value
    }
}

/// Accessor for a storage-backed [`Address`].
#[derive(Debug)]
pub struct StorageAddress {
    slot: U256,
    offset: u8,
    cached: OnceCell<Address>,
}

impl StorageAddress {
    /// Gets the underlying [`Address`] in persistent storage.
    pub fn get(&self) -> Address {
        **self
    }

    /// Gets the underlying [`Address`] in persistent storage.
    pub fn set(&mut self, value: Address) {
        unsafe { StorageCache::set::<20>(self.slot, self.offset.into(), value.into()) }
    }
}

impl StorageType for StorageAddress {
    const SIZE: u8 = 20;

    fn new(slot: U256, offset: u8) -> Self {
        Self {
            slot,
            offset,
            cached: OnceCell::new(),
        }
    }
}

impl SizedStorageType for StorageAddress {
    type Value = Address;

    fn set_exact(&mut self, value: Self::Value) {
        self.set(value);
    }

    fn erase(&mut self) {
        self.set(Self::Value::ZERO);
    }
}

impl Deref for StorageAddress {
    type Target = Address;

    fn deref(&self) -> &Self::Target {
        self.cached.get_or_init(|| unsafe {
            StorageCache::get::<20>(self.slot, self.offset.into()).into()
        })
    }
}

impl From<StorageAddress> for Address {
    fn from(value: StorageAddress) -> Self {
        *value
    }
}

/// Accessor for a storage-backed [`BlockNumber`].
#[derive(Debug)]
pub struct StorageBlockNumber {
    slot: U256,
    offset: u8,
    cached: OnceCell<BlockNumber>,
}

impl StorageBlockNumber {
    /// Gets the underlying [`BlockNumber`] in persistent storage.
    pub fn get(&self) -> BlockNumber {
        **self
    }

    /// Gets the underlying [`BlockNumber`] in persistent storage.
    pub fn set(&self, value: BlockNumber) {
        let value = FixedBytes::from(value.to_be_bytes());
        unsafe { StorageCache::set::<8>(self.slot, self.offset.into(), value) };
    }
}

impl StorageType for StorageBlockNumber {
    const SIZE: u8 = 8;

    fn new(slot: U256, offset: u8) -> Self {
        Self {
            slot,
            offset,
            cached: OnceCell::new(),
        }
    }
}

impl SizedStorageType for StorageBlockNumber {
    type Value = BlockNumber;

    fn set_exact(&mut self, value: Self::Value) {
        self.set(value);
    }

    fn erase(&mut self) {
        self.set(0);
    }
}

impl Deref for StorageBlockNumber {
    type Target = BlockNumber;

    fn deref(&self) -> &Self::Target {
        self.cached.get_or_init(|| unsafe {
            let data = StorageCache::get::<8>(self.slot, self.offset.into());
            transmute(data)
        })
    }
}

impl From<StorageBlockNumber> for BlockNumber {
    fn from(value: StorageBlockNumber) -> Self {
        *value
    }
}

/// Accessor for a storage-backed [`BlockHash`].
#[derive(Clone, Debug)]
pub struct StorageBlockHash {
    slot: U256,
    cached: OnceCell<BlockHash>,
}

impl StorageBlockHash {
    /// Gets the underlying [`BlockHash`] in persistent storage.
    pub fn get(&self) -> BlockHash {
        **self
    }

    /// Sets the underlying [`BlockHash`] in persistent storage.
    pub fn set(&mut self, value: BlockHash) {
        self.cached.take();
        _ = self.cached.set(value);
        StorageCache::set_word(self.slot, value)
    }
}

impl StorageType for StorageBlockHash {
    fn new(slot: U256, _offset: u8) -> Self {
        let cached = OnceCell::new();
        Self { slot, cached }
    }
}

impl SizedStorageType for StorageBlockHash {
    type Value = BlockHash;

    fn set_exact(&mut self, value: Self::Value) {
        self.set(value);
    }

    fn erase(&mut self) {
        self.set(Self::Value::ZERO);
    }
}

impl Deref for StorageBlockHash {
    type Target = BlockHash;

    fn deref(&self) -> &Self::Target {
        self.cached
            .get_or_init(|| StorageCache::get_word(self.slot))
    }
}

impl From<StorageBlockHash> for BlockHash {
    fn from(value: StorageBlockHash) -> Self {
        *value
    }
}
