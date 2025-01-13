// Copyright 2025-2026, Offchain Labs, Inc.
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

use crate::{
    host::{Host, HostAccess},
    hostio,
};
use alloy_primitives::{Address, BlockHash, BlockNumber, FixedBytes, Signed, Uint, B256, U256};
use alloy_sol_types::sol_data::{ByteCount, IntBitCount, SupportedFixedBytes, SupportedInt};
use core::{cell::OnceCell, marker::PhantomData, ops::Deref};

pub use array::StorageArray;
pub use bytes::{StorageBytes, StorageString};
pub use map::{StorageKey, StorageMap};
pub use traits::{
    Erase, GlobalStorage, SimpleStorageType, StorageGuard, StorageGuardMut, StorageType,
    TopLevelStorage,
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

impl<H: Host> GlobalStorage<H> for StorageCache {
    /// Retrieves a 32-byte EVM word from persistent storage.
    fn get_word(host: &H, key: U256) -> B256 {
        host.storage_load_bytes32(key)
    }

    /// Stores a 32-byte EVM word to persistent storage.
    ///
    /// # Safety
    ///
    /// May alias storage.
    unsafe fn set_word(host: &H, key: U256, value: B256) {
        host.storage_cache_bytes32(key, value)
    }
}

impl StorageCache {
    /// Flushes the VM cache, persisting all values to the EVM state trie.
    /// Note: this is used at the end of the [`entrypoint`] macro and is not typically called by user code.
    ///
    /// [`entrypoint`]: macro@stylus_proc::entrypoint
    pub fn flush() {
        unsafe { hostio::storage_flush_cache(false) }
    }

    /// Flushes and clears the VM cache, persisting all values to the EVM state trie.
    /// This is useful in cases of reentrancy to ensure cached values from one call context show up in another.
    pub fn clear() {
        unsafe { hostio::storage_flush_cache(true) }
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
            pub type $name<H> = StorageUint<H, $bits, $limbs>;

            #[doc = concat!("Accessor for a storage-backed [`alloy_primitives::aliases::I", stringify!($bits), "`].")]
            pub type $signed_name<H> = StorageSigned<H, $bits, $limbs>;
        )*
    };
}

macro_rules! alias_bytes {
    ($($name:ident, $bits:expr, $bytes:expr;)*) => {
        $(
            #[doc = concat!("Accessor for a storage-backed [`alloy_primitives::aliases::B", stringify!($bits), "`].")]
            pub type $name<H> = StorageFixedBytes<H, $bytes>;
        )*
    };
}

alias_ints! {
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
pub struct StorageUint<H, const B: usize, const L: usize>
where
    IntBitCount<B>: SupportedInt,
    H: Host,
{
    slot: U256,
    offset: u8,
    cached: OnceCell<Uint<B, L>>,
    __stylus_host: *const H,
}

impl<H, const B: usize, const L: usize> StorageUint<H, B, L>
where
    IntBitCount<B>: SupportedInt,
    H: Host,
{
    /// Gets the underlying [`alloy_primitives::Uint`] in persistent storage.
    pub fn get(&self) -> Uint<B, L> {
        **self
    }

    /// Sets the underlying [`alloy_primitives::Uint`] in persistent storage.
    pub fn set(&mut self, value: Uint<B, L>) {
        overwrite_cell(&mut self.cached, value);
        unsafe { Storage::set_uint(self.vm(), self.slot, self.offset.into(), value) };
    }
}

impl<H, const B: usize, const L: usize> HostAccess for StorageUint<H, B, L>
where
    IntBitCount<B>: SupportedInt,
    H: Host,
{
    type Host = H;
    fn vm(&self) -> &H {
        // SAFETY: Host is guaranteed to be valid and non-null for the lifetime of the storage
        // as injected by the Stylus entrypoint function.
        unsafe { &*self.__stylus_host }
    }
}

impl<H, const B: usize, const L: usize> StorageType<H> for StorageUint<H, B, L>
where
    IntBitCount<B>: SupportedInt,
    H: Host,
{
    type Wraps<'a>
        = Uint<B, L>
    where
        Self: 'a;
    type WrapsMut<'a>
        = StorageGuardMut<'a, Self>
    where
        Self: 'a;

    const SLOT_BYTES: usize = (B / 8);

    unsafe fn new(slot: U256, offset: u8, host: *const H) -> Self {
        debug_assert!(B <= 256);
        Self {
            slot,
            offset,
            cached: OnceCell::new(),
            __stylus_host: host,
        }
    }

    fn load<'s>(self) -> Self::Wraps<'s>
    where
        Self: 's,
    {
        self.get()
    }

    fn load_mut<'s>(self) -> Self::WrapsMut<'s> {
        StorageGuardMut::new(self)
    }
}

impl<'a, H, const B: usize, const L: usize> SimpleStorageType<'a, H> for StorageUint<H, B, L>
where
    IntBitCount<B>: SupportedInt,
    H: Host,
    Self: 'a,
{
    fn set_by_wrapped(&mut self, value: Self::Wraps<'a>) {
        self.set(value);
    }
}

impl<H, const B: usize, const L: usize> Erase<H> for StorageUint<H, B, L>
where
    IntBitCount<B>: SupportedInt,
    H: Host,
{
    fn erase(&mut self) {
        self.set(Self::Wraps::ZERO);
    }
}

impl<H, const B: usize, const L: usize> Deref for StorageUint<H, B, L>
where
    IntBitCount<B>: SupportedInt,
    H: Host,
{
    type Target = Uint<B, L>;

    fn deref(&self) -> &Self::Target {
        self.cached
            .get_or_init(|| unsafe { Storage::get_uint(self.vm(), self.slot, self.offset.into()) })
    }
}

impl<H, const B: usize, const L: usize> From<StorageUint<H, B, L>> for Uint<B, L>
where
    IntBitCount<B>: SupportedInt,
    H: Host,
{
    fn from(value: StorageUint<H, B, L>) -> Self {
        *value
    }
}

/// Accessor for a storage-backed [`Signed`].
///
/// Note: in the future `L` won't be needed.
// TODO: drop L after SupportedInt provides LIMBS (waiting for clarity reasons)
// https://github.com/rust-lang/rust/issues/76560
#[derive(Debug)]
pub struct StorageSigned<H, const B: usize, const L: usize>
where
    IntBitCount<B>: SupportedInt,
    H: Host,
{
    slot: U256,
    offset: u8,
    cached: OnceCell<Signed<B, L>>,
    __stylus_host: *const H,
}

impl<H, const B: usize, const L: usize> StorageSigned<H, B, L>
where
    IntBitCount<B>: SupportedInt,
    H: Host,
{
    /// Gets the underlying [`Signed`] in persistent storage.
    pub fn get(&self) -> Signed<B, L> {
        **self
    }

    /// Gets the underlying [`Signed`] in persistent storage.
    pub fn set(&mut self, value: Signed<B, L>) {
        overwrite_cell(&mut self.cached, value);
        unsafe { Storage::set_signed(self.vm(), self.slot, self.offset.into(), value) };
    }
}

impl<H, const B: usize, const L: usize> StorageType<H> for StorageSigned<H, B, L>
where
    IntBitCount<B>: SupportedInt,
    H: Host,
{
    type Wraps<'a>
        = Signed<B, L>
    where
        Self: 'a;
    type WrapsMut<'a>
        = StorageGuardMut<'a, Self>
    where
        Self: 'a;

    const SLOT_BYTES: usize = (B / 8);

    unsafe fn new(slot: U256, offset: u8, host: *const H) -> Self {
        Self {
            slot,
            offset,
            cached: OnceCell::new(),
            __stylus_host: host,
        }
    }

    fn load<'s>(self) -> Self::Wraps<'s>
    where
        Self: 's,
    {
        self.get()
    }

    fn load_mut<'s>(self) -> Self::WrapsMut<'s> {
        StorageGuardMut::new(self)
    }
}

impl<H, const B: usize, const L: usize> HostAccess for StorageSigned<H, B, L>
where
    IntBitCount<B>: SupportedInt,
    H: Host,
{
    type Host = H;
    fn vm(&self) -> &H {
        // SAFETY: Host is guaranteed to be valid and non-null for the lifetime of the storage
        // as injected by the Stylus entrypoint function.
        unsafe { &*self.__stylus_host }
    }
}

impl<'a, H, const B: usize, const L: usize> SimpleStorageType<'a, H> for StorageSigned<H, B, L>
where
    IntBitCount<B>: SupportedInt,
    H: Host,
    Self: 'a,
{
    fn set_by_wrapped(&mut self, value: Self::Wraps<'a>)
    where
        Self: 'a,
    {
        self.set(value);
    }
}

impl<H, const B: usize, const L: usize> Erase<H> for StorageSigned<H, B, L>
where
    IntBitCount<B>: SupportedInt,
    H: Host,
{
    fn erase(&mut self) {
        self.set(Self::Wraps::ZERO)
    }
}

impl<H, const B: usize, const L: usize> Deref for StorageSigned<H, B, L>
where
    IntBitCount<B>: SupportedInt,
    H: Host,
{
    type Target = Signed<B, L>;

    fn deref(&self) -> &Self::Target {
        self.cached.get_or_init(|| unsafe {
            Storage::get_signed(self.vm(), self.slot, self.offset.into())
        })
    }
}

impl<H, const B: usize, const L: usize> From<StorageSigned<H, B, L>> for Signed<B, L>
where
    IntBitCount<B>: SupportedInt,
    H: Host,
{
    fn from(value: StorageSigned<H, B, L>) -> Self {
        *value
    }
}

/// Accessor for a storage-backed [`FixedBytes`].
#[derive(Debug)]
pub struct StorageFixedBytes<H: Host, const N: usize> {
    slot: U256,
    offset: u8,
    cached: OnceCell<FixedBytes<N>>,
    __stylus_host: *const H,
}

impl<H: Host, const N: usize> StorageFixedBytes<H, N> {
    /// Gets the underlying [`FixedBytes`] in persistent storage.
    pub fn get(&self) -> FixedBytes<N> {
        **self
    }

    /// Gets the underlying [`FixedBytes`] in persistent storage.
    pub fn set(&mut self, value: FixedBytes<N>) {
        overwrite_cell(&mut self.cached, value);
        unsafe { Storage::set(self.vm(), self.slot, self.offset.into(), value) }
    }
}

impl<H, const N: usize> StorageType<H> for StorageFixedBytes<H, N>
where
    ByteCount<N>: SupportedFixedBytes,
    H: Host,
{
    type Wraps<'a>
        = FixedBytes<N>
    where
        Self: 'a;
    type WrapsMut<'a>
        = StorageGuardMut<'a, Self>
    where
        Self: 'a;

    const SLOT_BYTES: usize = N;

    unsafe fn new(slot: U256, offset: u8, host: *const H) -> Self {
        Self {
            slot,
            offset,
            cached: OnceCell::new(),
            __stylus_host: host,
        }
    }

    fn load<'s>(self) -> Self::Wraps<'s>
    where
        Self: 's,
    {
        self.get()
    }

    fn load_mut<'s>(self) -> Self::WrapsMut<'s> {
        StorageGuardMut::new(self)
    }
}

impl<H, const N: usize> HostAccess for StorageFixedBytes<H, N>
where
    H: Host,
{
    type Host = H;
    fn vm(&self) -> &H {
        // SAFETY: Host is guaranteed to be valid and non-null for the lifetime of the storage
        // as injected by the Stylus entrypoint function.
        unsafe { &*self.__stylus_host }
    }
}

impl<'a, H, const N: usize> SimpleStorageType<'a, H> for StorageFixedBytes<H, N>
where
    ByteCount<N>: SupportedFixedBytes,
    H: Host,
    Self: 'a,
{
    fn set_by_wrapped(&mut self, value: Self::Wraps<'a>) {
        self.set(value);
    }
}

impl<H, const N: usize> Erase<H> for StorageFixedBytes<H, N>
where
    ByteCount<N>: SupportedFixedBytes,
    H: Host,
{
    fn erase(&mut self) {
        self.set(Self::Wraps::ZERO)
    }
}

impl<H: Host, const N: usize> Deref for StorageFixedBytes<H, N> {
    type Target = FixedBytes<N>;

    fn deref(&self) -> &Self::Target {
        self.cached
            .get_or_init(|| unsafe { Storage::get(self.vm(), self.slot, self.offset.into()) })
    }
}

impl<H: Host, const N: usize> From<StorageFixedBytes<H, N>> for FixedBytes<N> {
    fn from(value: StorageFixedBytes<H, N>) -> Self {
        *value
    }
}

/// Accessor for a storage-backed [`bool`].
#[derive(Debug)]
pub struct StorageBool<H: Host> {
    slot: U256,
    offset: u8,
    cached: OnceCell<bool>,
    __stylus_host: *const H,
}

impl<H: Host> StorageBool<H> {
    /// Gets the underlying [`bool`] in persistent storage.
    pub fn get(&self) -> bool {
        **self
    }

    /// Gets the underlying [`bool`] in persistent storage.
    pub fn set(&mut self, value: bool) {
        overwrite_cell(&mut self.cached, value);
        unsafe { Storage::set_byte(self.vm(), self.slot, self.offset.into(), value as u8) }
    }
}

impl<H: Host> StorageType<H> for StorageBool<H> {
    type Wraps<'a>
        = bool
    where
        Self: 'a;
    type WrapsMut<'a>
        = StorageGuardMut<'a, Self>
    where
        Self: 'a;

    const SLOT_BYTES: usize = 1;

    unsafe fn new(slot: U256, offset: u8, host: *const H) -> Self {
        Self {
            slot,
            offset,
            cached: OnceCell::new(),
            __stylus_host: host,
        }
    }

    fn load<'s>(self) -> Self::Wraps<'s>
    where
        Self: 's,
    {
        self.get()
    }

    fn load_mut<'s>(self) -> Self::WrapsMut<'s> {
        StorageGuardMut::new(self)
    }
}

impl<H: Host> HostAccess for StorageBool<H> {
    type Host = H;
    fn vm(&self) -> &H {
        // SAFETY: Host is guaranteed to be valid and non-null for the lifetime of the storage
        // as injected by the Stylus entrypoint function.
        unsafe { &*self.__stylus_host }
    }
}

impl<'a, H: Host> SimpleStorageType<'a, H> for StorageBool<H>
where
    Self: 'a,
{
    fn set_by_wrapped(&mut self, value: Self::Wraps<'a>)
    where
        Self: 'a,
    {
        self.set(value);
    }
}

impl<H: Host> Erase<H> for StorageBool<H> {
    fn erase(&mut self) {
        self.set(false);
    }
}

impl<H: Host> Deref for StorageBool<H> {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        self.cached.get_or_init(|| unsafe {
            let data = Storage::get_byte(self.vm(), self.slot, self.offset.into());
            data != 0
        })
    }
}

impl<H: Host> From<StorageBool<H>> for bool {
    fn from(value: StorageBool<H>) -> Self {
        *value
    }
}

/// Accessor for a storage-backed [`Address`].
#[derive(Debug)]
pub struct StorageAddress<H: Host> {
    slot: U256,
    offset: u8,
    cached: OnceCell<Address>,
    __stylus_host: *const H,
}

impl<H: Host> StorageAddress<H> {
    /// Gets the underlying [`Address`] in persistent storage.
    pub fn get(&self) -> Address {
        **self
    }

    /// Gets the underlying [`Address`] in persistent storage.
    pub fn set(&mut self, value: Address) {
        overwrite_cell(&mut self.cached, value);
        unsafe { Storage::set::<20>(self.vm(), self.slot, self.offset.into(), value.into()) }
    }
}

impl<H: Host> StorageType<H> for StorageAddress<H> {
    type Wraps<'a>
        = Address
    where
        Self: 'a;
    type WrapsMut<'a>
        = StorageGuardMut<'a, Self>
    where
        Self: 'a;

    const SLOT_BYTES: usize = 20;

    unsafe fn new(slot: U256, offset: u8, host: *const H) -> Self {
        Self {
            slot,
            offset,
            cached: OnceCell::new(),
            __stylus_host: host,
        }
    }

    fn load<'s>(self) -> Self::Wraps<'s>
    where
        Self: 's,
    {
        self.get()
    }

    fn load_mut<'s>(self) -> Self::WrapsMut<'s> {
        StorageGuardMut::new(self)
    }
}

impl<H: Host> HostAccess for StorageAddress<H> {
    type Host = H;
    fn vm(&self) -> &H {
        // SAFETY: Host is guaranteed to be valid and non-null for the lifetime of the storage
        // as injected by the Stylus entrypoint function.
        unsafe { &*self.__stylus_host }
    }
}

impl<'a, H: Host> SimpleStorageType<'a, H> for StorageAddress<H>
where
    Self: 'a,
{
    fn set_by_wrapped(&mut self, value: Self::Wraps<'a>) {
        self.set(value);
    }
}

impl<H: Host> Erase<H> for StorageAddress<H> {
    fn erase(&mut self) {
        self.set(Self::Wraps::ZERO);
    }
}

impl<H: Host> Deref for StorageAddress<H> {
    type Target = Address;

    fn deref(&self) -> &Self::Target {
        self.cached.get_or_init(|| unsafe {
            Storage::get::<20>(self.vm(), self.slot, self.offset.into()).into()
        })
    }
}

impl<H: Host> From<StorageAddress<H>> for Address {
    fn from(value: StorageAddress<H>) -> Self {
        *value
    }
}

/// Accessor for a storage-backed [`BlockNumber`].
///
/// This storage type allows convenient and type-safe storage of a
/// [`BlockNumber`].
#[derive(Debug)]
pub struct StorageBlockNumber<H: Host> {
    slot: U256,
    offset: u8,
    cached: OnceCell<BlockNumber>,
    __stylus_host: *const H,
}

impl<H: Host> StorageBlockNumber<H> {
    /// Gets the underlying [`BlockNumber`] in persistent storage.
    pub fn get(&self) -> BlockNumber {
        **self
    }

    /// Sets the underlying [`BlockNumber`] in persistent storage.
    pub fn set(&mut self, value: BlockNumber) {
        overwrite_cell(&mut self.cached, value);
        let value = FixedBytes::from(value.to_be_bytes());
        unsafe { Storage::set::<8>(self.vm(), self.slot, self.offset.into(), value) };
    }
}

impl<H: Host> StorageType<H> for StorageBlockNumber<H> {
    type Wraps<'a>
        = BlockNumber
    where
        Self: 'a;
    type WrapsMut<'a>
        = StorageGuardMut<'a, Self>
    where
        Self: 'a;

    const SLOT_BYTES: usize = 8;

    unsafe fn new(slot: U256, offset: u8, host: *const H) -> Self {
        Self {
            slot,
            offset,
            cached: OnceCell::new(),
            __stylus_host: host,
        }
    }

    fn load<'s>(self) -> Self::Wraps<'s>
    where
        Self: 's,
    {
        self.get()
    }

    fn load_mut<'s>(self) -> Self::WrapsMut<'s> {
        StorageGuardMut::new(self)
    }
}

impl<H: Host> HostAccess for StorageBlockNumber<H> {
    type Host = H;
    fn vm(&self) -> &H {
        // SAFETY: Host is guaranteed to be valid and non-null for the lifetime of the storage
        // as injected by the Stylus entrypoint function.
        unsafe { &*self.__stylus_host }
    }
}

impl<'a, H: Host> SimpleStorageType<'a, H> for StorageBlockNumber<H>
where
    Self: 'a,
{
    fn set_by_wrapped(&mut self, value: Self::Wraps<'a>) {
        self.set(value);
    }
}

impl<H: Host> Erase<H> for StorageBlockNumber<H> {
    fn erase(&mut self) {
        self.set(0);
    }
}

impl<H: Host> Deref for StorageBlockNumber<H> {
    type Target = BlockNumber;

    fn deref(&self) -> &Self::Target {
        self.cached.get_or_init(|| unsafe {
            let data = Storage::get::<8>(self.vm(), self.slot, self.offset.into());
            u64::from_be_bytes(data.0)
        })
    }
}

impl<H: Host> From<StorageBlockNumber<H>> for BlockNumber {
    fn from(value: StorageBlockNumber<H>) -> Self {
        *value
    }
}

/// Accessor for a storage-backed [`BlockHash`].
///
/// This storage type allows convenient and type-safe storage of a
/// [`BlockHash`].
#[derive(Clone, Debug)]
pub struct StorageBlockHash<H: Host> {
    slot: U256,
    cached: OnceCell<BlockHash>,
    __stylus_host: *const H,
}

impl<H: Host> StorageBlockHash<H> {
    /// Gets the underlying [`BlockHash`] in persistent storage.
    pub fn get(&self) -> BlockHash {
        **self
    }

    /// Sets the underlying [`BlockHash`] in persistent storage.
    pub fn set(&mut self, value: BlockHash) {
        overwrite_cell(&mut self.cached, value);
        unsafe { Storage::set_word(self.vm(), self.slot, value) }
    }
}

impl<H: Host> StorageType<H> for StorageBlockHash<H> {
    type Wraps<'a>
        = BlockHash
    where
        Self: 'a;
    type WrapsMut<'a>
        = StorageGuardMut<'a, Self>
    where
        Self: 'a;

    unsafe fn new(slot: U256, _offset: u8, host: *const H) -> Self {
        let cached = OnceCell::new();
        Self {
            slot,
            cached,
            __stylus_host: host,
        }
    }

    fn load<'s>(self) -> Self::Wraps<'s>
    where
        Self: 's,
    {
        self.get()
    }

    fn load_mut<'s>(self) -> Self::WrapsMut<'s> {
        StorageGuardMut::new(self)
    }
}

impl<H: Host> HostAccess for StorageBlockHash<H> {
    type Host = H;
    fn vm(&self) -> &H {
        // SAFETY: Host is guaranteed to be valid and non-null for the lifetime of the storage
        // as injected by the Stylus entrypoint function.
        unsafe { &*self.__stylus_host }
    }
}

impl<'a, H: Host> SimpleStorageType<'a, H> for StorageBlockHash<H>
where
    Self: 'a,
{
    fn set_by_wrapped(&mut self, value: Self::Wraps<'a>) {
        self.set(value);
    }
}

impl<H: Host> Erase<H> for StorageBlockHash<H> {
    fn erase(&mut self) {
        self.set(Self::Wraps::ZERO);
    }
}

impl<H: Host> Deref for StorageBlockHash<H> {
    type Target = BlockHash;

    fn deref(&self) -> &Self::Target {
        self.cached
            .get_or_init(|| Storage::get_word(self.vm(), self.slot))
    }
}

impl<H: Host> From<StorageBlockHash<H>> for BlockHash {
    fn from(value: StorageBlockHash<H>) -> Self {
        *value
    }
}

/// We implement `StorageType` for `PhantomData` so that storage types can be generic.
impl<T, H: Host> StorageType<H> for PhantomData<T> {
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

    unsafe fn new(_slot: U256, _offset: u8, _host: *const H) -> Self {
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
