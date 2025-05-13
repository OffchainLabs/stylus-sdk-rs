// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Solidity compatible storage types and persistent storage access.
//!
//! The Stylus node software is composed of two, fully-composable virtual machines.
//! - The Stylus VM, which compiles WASM contracts built with SDKs like this one.
//! - The Ethereum Virtual Machine, which interprets EVM bytecode from languages like Solidity and Vyper.
//!
//! Though these two VMs differ in execution, they are backed by the same EVM State Trie.
//! This means that Stylus contracts have access to the same, key-value based persistent storage
//! familiar to Solidity devs.
//!
//! Because this resource is foreign to Rust, this module provides standard types and traits for
//! accessing state when writing programs. To protect the user, the Stylus SDK safeguards storage access
//! by leveraging Rust's borrow checker. It should never be possible to alias Storage without `unsafe` Rust,
//! eliminating entire classes of errors at compile time.
//!
//! Storage Operations are also cached by default, ensuring that efficient usage is clean and auditable.
//!
//! For a walkthrough of this module's features, please see [The Feature Overview][overview].
//!
//! [overview]: https://docs.arbitrum.io/stylus/reference/rust-sdk-guide#storage

use crate::{host::VM, hostio};
use alloy_primitives::{Address, BlockHash, BlockNumber, FixedBytes, Signed, Uint, B256, U256};
use alloy_sol_types::sol_data::{ByteCount, IntBitCount, SupportedFixedBytes, SupportedInt};
use cfg_if::cfg_if;
use core::{cell::OnceCell, marker::PhantomData, ops::Deref};
use stylus_core::*;

pub use array::StorageArray;
pub use bytes::{StorageBytes, StorageString};
pub use map::{StorageKey, StorageMap};
pub use traits::{
    Erase, GlobalStorage, SimpleStorageType, StorageGuard, StorageGuardMut, StorageType,
};
pub use vec::StorageVec;

mod array;
mod bytes;
mod map;
mod traits;
mod vec;

pub(crate) type Storage = StorageCache;

/// Global accessor to persistent storage that relies on VM-level caching.
///
/// [`LocalStorageCache`]: super::LocalStorageCache
pub struct StorageCache;

impl GlobalStorage for StorageCache {
    /// Retrieves a 32-byte EVM word from persistent storage.
    fn get_word(vm: VM, key: U256) -> B256 {
        cfg_if! {
            if #[cfg(not(feature = "stylus-test"))] {
                vm.storage_load_bytes32(key)
            } else {
                vm.host.storage_load_bytes32(key)
            }
        }
    }

    /// Stores a 32-byte EVM word to persistent storage.
    ///
    /// # Safety
    ///
    /// May alias storage.
    unsafe fn set_word(vm: VM, key: U256, value: B256) {
        cfg_if! {
            if #[cfg(not(feature = "stylus-test"))] {
                vm.storage_cache_bytes32(key, value)
            } else {
                vm.host.storage_cache_bytes32(key, value)
            }
        }
    }
}

/// Overwrites the value in a cell.
#[inline]
fn overwrite_cell<T>(cell: &mut OnceCell<T>, value: T) {
    cell.take();
    _ = cell.set(value);
}

macro_rules! alias_ints {
    ($($name:ident, $signed_name:ident, $bits:expr, $limbs:expr;)*) => {
        $(
            #[doc = concat!("Accessor for a storage-backed [`alloy_primitives::aliases::U", stringify!($bits), "`].")]
            pub type $name = StorageUint<$bits, $limbs>;

            #[doc = concat!("Accessor for a storage-backed [`alloy_primitives::aliases::I", stringify!($bits), "`].")]
            pub type $signed_name = StorageSigned<$bits, $limbs>;
        )*
    };
}

macro_rules! alias_bytes {
    ($($name:ident, $bits:expr, $bytes:expr;)*) => {
        $(
            #[doc = concat!("Accessor for a storage-backed [`alloy_primitives::aliases::B", stringify!($bits), "`].")]
            pub type $name = StorageFixedBytes<$bytes>;
        )*
    };
}

alias_ints! {
    StorageU8, StorageI8, 8, 1;
    StorageU16, StorageI16, 16, 1;
    StorageU32, StorageI32, 32, 1;
    StorageU64, StorageI64, 64, 1;
    StorageU96, StorageI96, 96, 2;
    StorageU128, StorageI128, 128, 2;
    StorageU160, StorageI160, 160, 3;
    StorageU192, StorageI192, 192, 3;
    StorageU256, StorageI256, 256, 4;
}

alias_bytes! {
    StorageB8, 8, 1;
    StorageB16, 16, 2;
    StorageB32, 32, 4;
    StorageB64, 64, 8;
    StorageB96, 96, 12;
    StorageB128, 128, 16;
    StorageB160, 160, 20;
    StorageB192, 192, 24;
    StorageB224, 224, 28;
    StorageB256, 256, 32;
}

/// Accessor for a storage-backed [`alloy_primitives::Uint`].
///
/// Note: in the future `L` won't be needed.
// TODO: drop L after SupportedInt provides LIMBS (waiting for clarity reasons)
// https://github.com/rust-lang/rust/issues/76560
#[derive(Debug)]
pub struct StorageUint<const B: usize, const L: usize>
where
    IntBitCount<B>: SupportedInt,
{
    slot: U256,
    offset: u8,
    cached: OnceCell<Uint<B, L>>,
    __stylus_host: VM,
}

impl<const B: usize, const L: usize> HostAccess for StorageUint<B, L>
where
    IntBitCount<B>: SupportedInt,
{
    fn vm(&self) -> &dyn stylus_core::Host {
        cfg_if! {
            if #[cfg(not(feature = "stylus-test"))] {
                &self.__stylus_host
            } else {
                self.__stylus_host.host.as_ref()
            }
        }
    }
}

#[cfg(feature = "stylus-test")]
impl<const B: usize, const L: usize, T> From<&T> for StorageUint<B, L>
where
    IntBitCount<B>: SupportedInt,
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

impl<const B: usize, const L: usize> StorageUint<B, L>
where
    IntBitCount<B>: SupportedInt,
{
    /// Gets the underlying [`alloy_primitives::Uint`] in persistent storage.
    pub fn get(&self) -> Uint<B, L> {
        **self
    }

    /// Sets the underlying [`alloy_primitives::Uint`] in persistent storage.
    pub fn set(&mut self, value: Uint<B, L>) {
        overwrite_cell(&mut self.cached, value);
        unsafe {
            Storage::set_uint(
                self.__stylus_host.clone(),
                self.slot,
                self.offset.into(),
                value,
            )
        };
    }
}

impl<const B: usize, const L: usize> StorageType for StorageUint<B, L>
where
    IntBitCount<B>: SupportedInt,
{
    type Wraps<'a> = Uint<B, L>;
    type WrapsMut<'a> = StorageGuardMut<'a, Self>;

    const SLOT_BYTES: usize = (B / 8);

    unsafe fn new(slot: U256, offset: u8, host: VM) -> Self {
        debug_assert!(B <= 256);
        Self {
            slot,
            offset,
            cached: OnceCell::new(),
            __stylus_host: host,
        }
    }

    fn load<'s>(self) -> Self::Wraps<'s> {
        self.get()
    }

    fn load_mut<'s>(self) -> Self::WrapsMut<'s> {
        StorageGuardMut::new(self)
    }
}

impl<'a, const B: usize, const L: usize> SimpleStorageType<'a> for StorageUint<B, L>
where
    IntBitCount<B>: SupportedInt,
{
    fn set_by_wrapped(&mut self, value: Self::Wraps<'a>) {
        self.set(value);
    }
}

impl<const B: usize, const L: usize> Erase for StorageUint<B, L>
where
    IntBitCount<B>: SupportedInt,
{
    fn erase(&mut self) {
        self.set(Self::Wraps::ZERO);
    }
}

impl<const B: usize, const L: usize> Deref for StorageUint<B, L>
where
    IntBitCount<B>: SupportedInt,
{
    type Target = Uint<B, L>;

    fn deref(&self) -> &Self::Target {
        self.cached.get_or_init(|| unsafe {
            Storage::get_uint(self.__stylus_host.clone(), self.slot, self.offset.into())
        })
    }
}

impl<const B: usize, const L: usize> From<StorageUint<B, L>> for Uint<B, L>
where
    IntBitCount<B>: SupportedInt,
{
    fn from(value: StorageUint<B, L>) -> Self {
        *value
    }
}

/// Accessor for a storage-backed [`Signed`].
///
/// Note: in the future `L` won't be needed.
// TODO: drop L after SupportedInt provides LIMBS (waiting for clarity reasons)
// https://github.com/rust-lang/rust/issues/76560
#[derive(Debug)]
pub struct StorageSigned<const B: usize, const L: usize>
where
    IntBitCount<B>: SupportedInt,
{
    slot: U256,
    offset: u8,
    cached: OnceCell<Signed<B, L>>,
    __stylus_host: VM,
}

impl<const B: usize, const L: usize> HostAccess for StorageSigned<B, L>
where
    IntBitCount<B>: SupportedInt,
{
    fn vm(&self) -> &dyn stylus_core::Host {
        cfg_if! {
            if #[cfg(not(feature = "stylus-test"))] {
                &self.__stylus_host
            } else {
                self.__stylus_host.host.as_ref()
            }
        }
    }
}

#[cfg(feature = "stylus-test")]
impl<const B: usize, const L: usize, T> From<&T> for StorageSigned<B, L>
where
    IntBitCount<B>: SupportedInt,
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

impl<const B: usize, const L: usize> StorageSigned<B, L>
where
    IntBitCount<B>: SupportedInt,
{
    /// Gets the underlying [`Signed`] in persistent storage.
    pub fn get(&self) -> Signed<B, L> {
        **self
    }

    /// Gets the underlying [`Signed`] in persistent storage.
    pub fn set(&mut self, value: Signed<B, L>) {
        overwrite_cell(&mut self.cached, value);
        unsafe {
            Storage::set_signed(
                self.__stylus_host.clone(),
                self.slot,
                self.offset.into(),
                value,
            )
        };
    }
}

impl<const B: usize, const L: usize> StorageType for StorageSigned<B, L>
where
    IntBitCount<B>: SupportedInt,
{
    type Wraps<'a> = Signed<B, L>;
    type WrapsMut<'a> = StorageGuardMut<'a, Self>;

    const SLOT_BYTES: usize = (B / 8);

    unsafe fn new(slot: U256, offset: u8, host: VM) -> Self {
        Self {
            slot,
            offset,
            cached: OnceCell::new(),
            __stylus_host: host,
        }
    }

    fn load<'s>(self) -> Self::Wraps<'s> {
        self.get()
    }

    fn load_mut<'s>(self) -> Self::WrapsMut<'s> {
        StorageGuardMut::new(self)
    }
}

impl<'a, const B: usize, const L: usize> SimpleStorageType<'a> for StorageSigned<B, L>
where
    IntBitCount<B>: SupportedInt,
{
    fn set_by_wrapped(&mut self, value: Self::Wraps<'a>) {
        self.set(value);
    }
}

impl<const B: usize, const L: usize> Erase for StorageSigned<B, L>
where
    IntBitCount<B>: SupportedInt,
{
    fn erase(&mut self) {
        self.set(Self::Wraps::ZERO)
    }
}

impl<const B: usize, const L: usize> Deref for StorageSigned<B, L>
where
    IntBitCount<B>: SupportedInt,
{
    type Target = Signed<B, L>;

    fn deref(&self) -> &Self::Target {
        self.cached.get_or_init(|| unsafe {
            Storage::get_signed(self.__stylus_host.clone(), self.slot, self.offset.into())
        })
    }
}

impl<const B: usize, const L: usize> From<StorageSigned<B, L>> for Signed<B, L>
where
    IntBitCount<B>: SupportedInt,
{
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
    __stylus_host: VM,
}

impl<const N: usize> HostAccess for StorageFixedBytes<N> {
    fn vm(&self) -> &dyn stylus_core::Host {
        cfg_if! {
            if #[cfg(not(feature = "stylus-test"))] {
                &self.__stylus_host
            } else {
                self.__stylus_host.host.as_ref()
            }
        }
    }
}

impl<const N: usize> StorageFixedBytes<N> {
    /// Gets the underlying [`FixedBytes`] in persistent storage.
    pub fn get(&self) -> FixedBytes<N> {
        **self
    }

    /// Gets the underlying [`FixedBytes`] in persistent storage.
    pub fn set(&mut self, value: FixedBytes<N>) {
        overwrite_cell(&mut self.cached, value);
        unsafe {
            Storage::set(
                self.__stylus_host.clone(),
                self.slot,
                self.offset.into(),
                value,
            )
        }
    }
}

impl<const N: usize> StorageType for StorageFixedBytes<N>
where
    ByteCount<N>: SupportedFixedBytes,
{
    type Wraps<'a> = FixedBytes<N>;
    type WrapsMut<'a> = StorageGuardMut<'a, Self>;

    const SLOT_BYTES: usize = N;

    unsafe fn new(slot: U256, offset: u8, host: VM) -> Self {
        Self {
            slot,
            offset,
            cached: OnceCell::new(),
            __stylus_host: host,
        }
    }

    fn load<'s>(self) -> Self::Wraps<'s> {
        self.get()
    }

    fn load_mut<'s>(self) -> Self::WrapsMut<'s> {
        StorageGuardMut::new(self)
    }
}

#[cfg(feature = "stylus-test")]
impl<const N: usize, T> From<&T> for StorageFixedBytes<N>
where
    ByteCount<N>: SupportedFixedBytes,
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

impl<'a, const N: usize> SimpleStorageType<'a> for StorageFixedBytes<N>
where
    ByteCount<N>: SupportedFixedBytes,
{
    fn set_by_wrapped(&mut self, value: Self::Wraps<'a>) {
        self.set(value);
    }
}

impl<const N: usize> Erase for StorageFixedBytes<N>
where
    ByteCount<N>: SupportedFixedBytes,
{
    fn erase(&mut self) {
        self.set(Self::Wraps::ZERO)
    }
}

impl<const N: usize> Deref for StorageFixedBytes<N> {
    type Target = FixedBytes<N>;

    fn deref(&self) -> &Self::Target {
        self.cached.get_or_init(|| unsafe {
            Storage::get(self.__stylus_host.clone(), self.slot, self.offset.into())
        })
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
    __stylus_host: VM,
}

impl HostAccess for StorageBool {
    fn vm(&self) -> &dyn stylus_core::Host {
        cfg_if! {
            if #[cfg(not(feature = "stylus-test"))] {
                &self.__stylus_host
            } else {
                self.__stylus_host.host.as_ref()
            }
        }
    }
}

#[cfg(feature = "stylus-test")]
impl<T> From<&T> for StorageBool
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

impl StorageBool {
    /// Gets the underlying [`bool`] in persistent storage.
    pub fn get(&self) -> bool {
        **self
    }

    /// Gets the underlying [`bool`] in persistent storage.
    pub fn set(&mut self, value: bool) {
        overwrite_cell(&mut self.cached, value);
        unsafe {
            Storage::set_byte(
                self.__stylus_host.clone(),
                self.slot,
                self.offset.into(),
                value as u8,
            )
        }
    }
}

impl StorageType for StorageBool {
    type Wraps<'a> = bool;
    type WrapsMut<'a> = StorageGuardMut<'a, Self>;

    const SLOT_BYTES: usize = 1;

    unsafe fn new(slot: U256, offset: u8, host: VM) -> Self {
        Self {
            slot,
            offset,
            cached: OnceCell::new(),
            __stylus_host: host,
        }
    }

    fn load<'s>(self) -> Self::Wraps<'s> {
        self.get()
    }

    fn load_mut<'s>(self) -> Self::WrapsMut<'s> {
        StorageGuardMut::new(self)
    }
}

impl<'a> SimpleStorageType<'a> for StorageBool {
    fn set_by_wrapped(&mut self, value: Self::Wraps<'a>) {
        self.set(value);
    }
}

impl Erase for StorageBool {
    fn erase(&mut self) {
        self.set(false);
    }
}

impl Deref for StorageBool {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        self.cached.get_or_init(|| unsafe {
            let data = Storage::get_byte(self.__stylus_host.clone(), self.slot, self.offset.into());
            data != 0
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
    __stylus_host: VM,
}

impl HostAccess for StorageAddress {
    fn vm(&self) -> &dyn stylus_core::Host {
        cfg_if! {
            if #[cfg(not(feature = "stylus-test"))] {
                &self.__stylus_host
            } else {
                self.__stylus_host.host.as_ref()
            }
        }
    }
}

#[cfg(feature = "stylus-test")]
impl<T> From<&T> for StorageAddress
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

impl StorageAddress {
    /// Gets the underlying [`Address`] in persistent storage.
    pub fn get(&self) -> Address {
        **self
    }

    /// Gets the underlying [`Address`] in persistent storage.
    pub fn set(&mut self, value: Address) {
        overwrite_cell(&mut self.cached, value);
        unsafe {
            Storage::set::<20>(
                self.__stylus_host.clone(),
                self.slot,
                self.offset.into(),
                value.into(),
            )
        }
    }
}

impl StorageType for StorageAddress {
    type Wraps<'a> = Address;
    type WrapsMut<'a> = StorageGuardMut<'a, Self>;

    const SLOT_BYTES: usize = 20;

    unsafe fn new(slot: U256, offset: u8, host: VM) -> Self {
        Self {
            slot,
            offset,
            cached: OnceCell::new(),
            __stylus_host: host,
        }
    }

    fn load<'s>(self) -> Self::Wraps<'s> {
        self.get()
    }

    fn load_mut<'s>(self) -> Self::WrapsMut<'s> {
        StorageGuardMut::new(self)
    }
}

impl<'a> SimpleStorageType<'a> for StorageAddress {
    fn set_by_wrapped(&mut self, value: Self::Wraps<'a>) {
        self.set(value);
    }
}

impl Erase for StorageAddress {
    fn erase(&mut self) {
        self.set(Self::Wraps::ZERO);
    }
}

impl Deref for StorageAddress {
    type Target = Address;

    fn deref(&self) -> &Self::Target {
        self.cached.get_or_init(|| unsafe {
            Storage::get::<20>(self.__stylus_host.clone(), self.slot, self.offset.into()).into()
        })
    }
}

impl From<StorageAddress> for Address {
    fn from(value: StorageAddress) -> Self {
        *value
    }
}

/// Accessor for a storage-backed [`BlockNumber`].
///
/// This storage type allows convenient and type-safe storage of a
/// [`BlockNumber`].
#[derive(Debug)]
pub struct StorageBlockNumber {
    slot: U256,
    offset: u8,
    cached: OnceCell<BlockNumber>,
    __stylus_host: VM,
}

impl HostAccess for StorageBlockNumber {
    fn vm(&self) -> &dyn stylus_core::Host {
        cfg_if! {
            if #[cfg(not(feature = "stylus-test"))] {
                &self.__stylus_host
            } else {
                self.__stylus_host.host.as_ref()
            }
        }
    }
}

#[cfg(feature = "stylus-test")]
impl<T> From<&T> for StorageBlockNumber
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

impl StorageBlockNumber {
    /// Gets the underlying [`BlockNumber`] in persistent storage.
    pub fn get(&self) -> BlockNumber {
        **self
    }

    /// Sets the underlying [`BlockNumber`] in persistent storage.
    pub fn set(&mut self, value: BlockNumber) {
        overwrite_cell(&mut self.cached, value);
        let value = FixedBytes::from(value.to_be_bytes());
        unsafe {
            Storage::set::<8>(
                self.__stylus_host.clone(),
                self.slot,
                self.offset.into(),
                value,
            )
        };
    }
}

impl StorageType for StorageBlockNumber {
    type Wraps<'a> = BlockNumber;
    type WrapsMut<'a> = StorageGuardMut<'a, Self>;

    const SLOT_BYTES: usize = 8;

    unsafe fn new(slot: U256, offset: u8, host: VM) -> Self {
        Self {
            slot,
            offset,
            cached: OnceCell::new(),
            __stylus_host: host,
        }
    }

    fn load<'s>(self) -> Self::Wraps<'s> {
        self.get()
    }

    fn load_mut<'s>(self) -> Self::WrapsMut<'s> {
        StorageGuardMut::new(self)
    }
}

impl<'a> SimpleStorageType<'a> for StorageBlockNumber {
    fn set_by_wrapped(&mut self, value: Self::Wraps<'a>) {
        self.set(value);
    }
}

impl Erase for StorageBlockNumber {
    fn erase(&mut self) {
        self.set(0);
    }
}

impl Deref for StorageBlockNumber {
    type Target = BlockNumber;

    fn deref(&self) -> &Self::Target {
        self.cached.get_or_init(|| unsafe {
            let data = Storage::get::<8>(self.__stylus_host.clone(), self.slot, self.offset.into());
            u64::from_be_bytes(data.0)
        })
    }
}

impl From<StorageBlockNumber> for BlockNumber {
    fn from(value: StorageBlockNumber) -> Self {
        *value
    }
}

/// Accessor for a storage-backed [`BlockHash`].
///
/// This storage type allows convenient and type-safe storage of a
/// [`BlockHash`].
#[derive(Clone, Debug)]
pub struct StorageBlockHash {
    slot: U256,
    cached: OnceCell<BlockHash>,
    __stylus_host: VM,
}

impl HostAccess for StorageBlockHash {
    fn vm(&self) -> &dyn stylus_core::Host {
        cfg_if! {
            if #[cfg(not(feature = "stylus-test"))] {
                &self.__stylus_host
            } else {
                self.__stylus_host.host.as_ref()
            }
        }
    }
}

#[cfg(feature = "stylus-test")]
impl<T> From<&T> for StorageBlockHash
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

impl StorageBlockHash {
    /// Gets the underlying [`BlockHash`] in persistent storage.
    pub fn get(&self) -> BlockHash {
        **self
    }

    /// Sets the underlying [`BlockHash`] in persistent storage.
    pub fn set(&mut self, value: BlockHash) {
        overwrite_cell(&mut self.cached, value);
        unsafe { Storage::set_word(self.__stylus_host.clone(), self.slot, value) }
    }
}

impl StorageType for StorageBlockHash {
    type Wraps<'a> = BlockHash;
    type WrapsMut<'a> = StorageGuardMut<'a, Self>;

    unsafe fn new(slot: U256, _offset: u8, host: VM) -> Self {
        let cached = OnceCell::new();
        Self {
            slot,
            cached,
            __stylus_host: host,
        }
    }

    fn load<'s>(self) -> Self::Wraps<'s> {
        self.get()
    }

    fn load_mut<'s>(self) -> Self::WrapsMut<'s> {
        StorageGuardMut::new(self)
    }
}

impl<'a> SimpleStorageType<'a> for StorageBlockHash {
    fn set_by_wrapped(&mut self, value: Self::Wraps<'a>) {
        self.set(value);
    }
}

impl Erase for StorageBlockHash {
    fn erase(&mut self) {
        self.set(Self::Wraps::ZERO);
    }
}

impl Deref for StorageBlockHash {
    type Target = BlockHash;

    fn deref(&self) -> &Self::Target {
        self.cached
            .get_or_init(|| Storage::get_word(self.__stylus_host.clone(), self.slot))
    }
}

impl From<StorageBlockHash> for BlockHash {
    fn from(value: StorageBlockHash) -> Self {
        *value
    }
}

/// We implement `StorageType` for `PhantomData` so that storage types can be generic.
impl<T> StorageType for PhantomData<T> {
    type Wraps<'a>
        = Self
    where
        Self: 'a;
    type WrapsMut<'a>
        = Self
    where
        Self: 'a;

    const REQUIRED_SLOTS: usize = 0;
    const SLOT_BYTES: usize = 0;

    unsafe fn new(_slot: U256, _offset: u8, _host: VM) -> Self {
        Self
    }

    fn load<'s>(self) -> Self::Wraps<'s>
    where
        Self: 's,
    {
        self
    }

    fn load_mut<'s>(self) -> Self::WrapsMut<'s>
    where
        Self: 's,
    {
        self
    }
}
