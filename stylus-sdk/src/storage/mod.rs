// Copyright 2023, Offchain Labs, Inc.
// For license information, see https://github.com/OffchainLabs/nitro/blob/master/LICENSE

use alloy_primitives::{Address, BlockHash, BlockNumber, FixedBytes, Signed, Uint, U256};
use std::mem::transmute;

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
#[derive(Clone, Copy, Debug)]
pub struct StorageUint<const B: usize, const L: usize> {
    slot: U256,
    offset: u8,
}

impl<const B: usize, const L: usize> StorageUint<B, L> {
    /// Gets the underlying [`Uint`] in persistent storage.
    pub fn get(&self) -> Uint<B, L> {
        unsafe { StorageCache::get_uint(self.slot, self.offset.into()) }
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
        Self { slot, offset }
    }
}

impl<const B: usize, const L: usize> SizedStorageType for StorageUint<B, L> {
    type Value = Uint<B, L>;

    fn set_exact(&mut self, value: Self::Value) {
        self.set(value);
    }
}

/// Accessor for a storage-backed [`Signed`].
#[derive(Clone, Copy, Debug)]
pub struct StorageSigned<const B: usize, const L: usize> {
    slot: U256,
    offset: u8,
}

impl<const B: usize, const L: usize> StorageSigned<B, L> {
    /// Gets the underlying [`Signed`] in persistent storage.
    pub fn get(&self) -> Signed<B, L> {
        unsafe { StorageCache::get_signed(self.slot, self.offset.into()) }
    }

    /// Gets the underlying [`Signed`] in persistent storage.
    pub fn set(&mut self, value: Signed<B, L>) {
        unsafe { StorageCache::set_signed(self.slot, self.offset.into(), value) };
    }
}

impl<const B: usize, const L: usize> StorageType for StorageSigned<B, L> {
    const SIZE: u8 = (B / 8) as u8;

    fn new(slot: U256, offset: u8) -> Self {
        Self { slot, offset }
    }
}

impl<const B: usize, const L: usize> SizedStorageType for StorageSigned<B, L> {
    type Value = Signed<B, L>;

    fn set_exact(&mut self, value: Self::Value) {
        self.set(value);
    }
}

/// Accessor for a storage-backed [`FixedBytes`].
#[derive(Clone, Copy, Debug)]
pub struct StorageFixedBytes<const N: usize> {
    slot: U256,
    offset: u8,
}

impl<const N: usize> StorageFixedBytes<N> {
    /// Gets the underlying [`FixedBytes`] in persistent storage.
    pub fn get(&self) -> FixedBytes<N> {
        unsafe { StorageCache::get(self.slot, self.offset.into()) }
    }

    /// Gets the underlying [`FixedBytes`] in persistent storage.
    pub fn set(&mut self, value: FixedBytes<N>) {
        unsafe { StorageCache::set(self.slot, self.offset.into(), value) }
    }
}

impl<const N: usize> StorageType for StorageFixedBytes<N> {
    const SIZE: u8 = N as u8;

    fn new(slot: U256, offset: u8) -> Self {
        Self { slot, offset }
    }
}

impl<const N: usize> SizedStorageType for StorageFixedBytes<N> {
    type Value = FixedBytes<N>;

    fn set_exact(&mut self, value: Self::Value) {
        self.set(value);
    }
}

/// Accessor for a storage-backed [`bool`].
#[derive(Clone, Copy, Debug)]
pub struct StorageBool {
    slot: U256,
    offset: u8,
}

impl StorageBool {
    /// Gets the underlying [`bool`] in persistent storage.
    pub fn get(&self) -> bool {
        let data = unsafe { StorageCache::get::<1>(self.slot, self.offset.into()) };
        data.as_slice()[0] != 0
    }

    /// Gets the underlying [`bool`] in persistent storage.
    pub fn set(&mut self, value: bool) {
        let value = value.then_some(1).unwrap_or_default();
        let fixed = FixedBytes::from_slice(&[value]);
        unsafe { StorageCache::set::<1>(self.slot, self.offset.into(), fixed) }
    }
}

impl StorageType for StorageBool {
    const SIZE: u8 = 1;

    fn new(slot: U256, offset: u8) -> Self {
        Self { slot, offset }
    }
}

impl SizedStorageType for StorageBool {
    type Value = bool;

    fn set_exact(&mut self, value: Self::Value) {
        self.set(value);
    }
}

/// Accessor for a storage-backed [`Address`].
#[derive(Clone, Copy, Debug)]
pub struct StorageAddress {
    slot: U256,
    offset: u8,
}

impl StorageAddress {
    /// Gets the underlying [`Address`] in persistent storage.
    pub fn get(&self) -> Address {
        let data = unsafe { StorageCache::get::<20>(self.slot, self.offset.into()) };
        Address::from(data)
    }

    /// Gets the underlying [`Address`] in persistent storage.
    pub fn set(&mut self, value: Address) {
        unsafe { StorageCache::set::<20>(self.slot, self.offset.into(), value.into()) }
    }
}

impl StorageType for StorageAddress {
    const SIZE: u8 = 20;

    fn new(slot: U256, offset: u8) -> Self {
        Self { slot, offset }
    }
}

impl SizedStorageType for StorageAddress {
    type Value = Address;

    fn set_exact(&mut self, value: Self::Value) {
        self.set(value);
    }
}

/// Accessor for a storage-backed [`BlockNumber`].
#[derive(Clone, Copy, Debug)]
pub struct StorageBlockNumber {
    slot: U256,
    offset: u8,
}

impl StorageBlockNumber {
    /// Gets the underlying [`BlockNumber`] in persistent storage.
    pub fn get(&self) -> BlockNumber {
        let data = unsafe { StorageCache::get::<8>(self.slot, self.offset.into()) };
        unsafe { transmute(data) }
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
        Self { slot, offset }
    }
}

impl SizedStorageType for StorageBlockNumber {
    type Value = BlockNumber;

    fn set_exact(&mut self, value: Self::Value) {
        self.set(value);
    }
}

/// Accessor for a storage-backed [`BlockHash`].
#[derive(Clone, Copy, Debug)]
pub struct StorageBlockHash {
    slot: U256,
}

impl StorageBlockHash {
    /// Gets the underlying [`BlockHash`] in persistent storage.
    pub fn get(&self) -> BlockHash {
        StorageCache::get_word(self.slot)
    }

    /// Sets the underlying [`BlockHash`] in persistent storage.
    pub fn set(&mut self, value: BlockHash) {
        StorageCache::set_word(self.slot, value)
    }
}

impl StorageType for StorageBlockHash {
    fn new(slot: U256, _offset: u8) -> Self {
        Self { slot }
    }
}

impl SizedStorageType for StorageBlockHash {
    type Value = BlockHash;

    fn set_exact(&mut self, value: Self::Value) {
        self.set(value);
    }
}
