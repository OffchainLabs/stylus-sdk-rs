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

use crate::host::Host;
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

impl GlobalStorage for StorageCache {
    /// Retrieves a 32-byte EVM word from persistent storage.
    fn get_word(key: U256) -> B256 {
        B256::ZERO
    }

    /// Stores a 32-byte EVM word to persistent storage.
    ///
    /// # Safety
    ///
    /// May alias storage.
    unsafe fn set_word(key: U256, value: B256) {}
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
            pub type $name<'a, H: Host> = StorageUint<'a, $bits, $limbs, H>;

            #[doc = concat!("Accessor for a storage-backed [`alloy_primitives::aliases::I", stringify!($bits), "`].")]
            pub type $signed_name<'a, H: Host> = StorageSigned<'a, H, $bits, $limbs>;
        )*
    };
}

macro_rules! alias_bytes {
    ($($name:ident, $bits:expr, $bytes:expr;)*) => {
        $(
            #[doc = concat!("Accessor for a storage-backed [`alloy_primitives::aliases::B", stringify!($bits), "`].")]
            pub type $name<'a, H: Host> = StorageFixedBytes<'a, H, $bytes>;
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
pub struct StorageUint<'a, const B: usize, const L: usize, H: Host>
where
    IntBitCount<B>: SupportedInt,
{
    slot: U256,
    offset: u8,
    cached: OnceCell<Uint<B, L>>,
    host: &'a H,
}

impl<'a, const B: usize, const L: usize, H: Host> StorageUint<'_, B, L, H>
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
        unsafe { Storage::set_uint(self.slot, self.offset.into(), value) };
    }
}

impl<'b, const B: usize, const L: usize, H: Host> StorageType<'b, H> for StorageUint<'b, B, L, H>
where
    IntBitCount<B>: SupportedInt,
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

    unsafe fn new(slot: U256, offset: u8, host: &'b H) -> Self {
        debug_assert!(B <= 256);
        Self {
            slot,
            offset,
            cached: OnceCell::new(),
            host,
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

impl<'a, const B: usize, const L: usize, H: Host> SimpleStorageType<'a, H>
    for StorageUint<'a, B, L, H>
where
    IntBitCount<B>: SupportedInt,
{
    fn set_by_wrapped(&mut self, value: Self::Wraps<'a>) {
        self.set(value);
    }
}

impl<'a, const B: usize, const L: usize, H: Host> Erase<'a, H> for StorageUint<'a, B, L, H>
where
    IntBitCount<B>: SupportedInt,
{
    fn erase(&mut self) {
        self.set(Self::Wraps::ZERO);
    }
}

impl<'a, const B: usize, const L: usize, H: Host> Deref for StorageUint<'a, B, L, H>
where
    IntBitCount<B>: SupportedInt,
{
    type Target = Uint<B, L>;

    fn deref(&self) -> &Self::Target {
        self.cached
            .get_or_init(|| unsafe { Storage::get_uint(self.slot, self.offset.into()) })
    }
}

impl<'a, const B: usize, const L: usize, H: Host> From<StorageUint<'a, B, L, H>> for Uint<B, L>
where
    IntBitCount<B>: SupportedInt,
{
    fn from(value: StorageUint<'a, B, L, H>) -> Self {
        *value
    }
}

/// Accessor for a storage-backed [`Signed`].
///
/// Note: in the future `L` won't be needed.
// TODO: drop L after SupportedInt provides LIMBS (waiting for clarity reasons)
// https://github.com/rust-lang/rust/issues/76560
#[derive(Debug)]
pub struct StorageSigned<'a, H: Host, const B: usize, const L: usize>
where
    IntBitCount<B>: SupportedInt,
{
    slot: U256,
    offset: u8,
    cached: OnceCell<Signed<B, L>>,
    host: &'a H,
}

impl<'a, H: Host, const B: usize, const L: usize> StorageSigned<'a, H, B, L>
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
        unsafe { Storage::set_signed(self.slot, self.offset.into(), value) };
    }
}

impl<'b, H: Host, const B: usize, const L: usize> StorageType<'b, H> for StorageSigned<'b, H, B, L>
where
    IntBitCount<B>: SupportedInt,
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

    unsafe fn new(slot: U256, offset: u8, host: &'b H) -> Self {
        Self {
            slot,
            offset,
            cached: OnceCell::new(),
            host,
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

impl<'a, H: Host, const B: usize, const L: usize> SimpleStorageType<'a, H>
    for StorageSigned<'a, H, B, L>
where
    IntBitCount<B>: SupportedInt,
{
    fn set_by_wrapped(&mut self, value: Self::Wraps<'a>) {
        self.set(value);
    }
}

impl<'a, H: Host, const B: usize, const L: usize> Erase<'a, H> for StorageSigned<'a, H, B, L>
where
    IntBitCount<B>: SupportedInt,
{
    fn erase(&mut self) {
        self.set(Self::Wraps::ZERO)
    }
}

impl<'a, H: Host, const B: usize, const L: usize> Deref for StorageSigned<'a, H, B, L>
where
    IntBitCount<B>: SupportedInt,
{
    type Target = Signed<B, L>;

    fn deref(&self) -> &Self::Target {
        self.cached
            .get_or_init(|| unsafe { Storage::get_signed(self.slot, self.offset.into()) })
    }
}

impl<'a, H: Host, const B: usize, const L: usize> From<StorageSigned<'a, H, B, L>> for Signed<B, L>
where
    IntBitCount<B>: SupportedInt,
{
    fn from(value: StorageSigned<'a, H, B, L>) -> Self {
        *value
    }
}

/// Accessor for a storage-backed [`FixedBytes`].
#[derive(Debug)]
pub struct StorageFixedBytes<'a, H: Host, const N: usize> {
    slot: U256,
    offset: u8,
    cached: OnceCell<FixedBytes<N>>,
    host: &'a H,
}

impl<'a, H: Host, const N: usize> StorageFixedBytes<'a, H, N> {
    /// Gets the underlying [`FixedBytes`] in persistent storage.
    pub fn get(&self) -> FixedBytes<N> {
        **self
    }

    /// Gets the underlying [`FixedBytes`] in persistent storage.
    pub fn set(&mut self, value: FixedBytes<N>) {
        overwrite_cell(&mut self.cached, value);
        unsafe { Storage::set(self.slot, self.offset.into(), value) }
    }
}

impl<'b, H: Host, const N: usize> StorageType<'b, H> for StorageFixedBytes<'b, H, N>
where
    ByteCount<N>: SupportedFixedBytes,
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

    unsafe fn new(slot: U256, offset: u8, host: &'b H) -> Self {
        Self {
            slot,
            offset,
            cached: OnceCell::new(),
            host,
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

impl<'a, H: Host, const N: usize> SimpleStorageType<'a, H> for StorageFixedBytes<'a, H, N>
where
    ByteCount<N>: SupportedFixedBytes,
{
    fn set_by_wrapped(&mut self, value: Self::Wraps<'a>) {
        self.set(value);
    }
}

impl<'a, H: Host, const N: usize> Erase<'a, H> for StorageFixedBytes<'a, H, N>
where
    ByteCount<N>: SupportedFixedBytes,
{
    fn erase(&mut self) {
        self.set(Self::Wraps::ZERO)
    }
}

impl<'a, H: Host, const N: usize> Deref for StorageFixedBytes<'a, H, N> {
    type Target = FixedBytes<N>;

    fn deref(&self) -> &Self::Target {
        self.cached
            .get_or_init(|| unsafe { Storage::get(self.slot, self.offset.into()) })
    }
}

impl<'a, H: Host, const N: usize> From<StorageFixedBytes<'a, H, N>> for FixedBytes<N> {
    fn from(value: StorageFixedBytes<'a, H, N>) -> Self {
        *value
    }
}

/// Accessor for a storage-backed [`bool`].
#[derive(Debug)]
pub struct StorageBool<'a, H: Host> {
    slot: U256,
    offset: u8,
    cached: OnceCell<bool>,
    host: &'a H,
}

impl<'a, H: Host> StorageBool<'a, H> {
    /// Gets the underlying [`bool`] in persistent storage.
    pub fn get(&self) -> bool {
        **self
    }

    /// Gets the underlying [`bool`] in persistent storage.
    pub fn set(&mut self, value: bool) {
        overwrite_cell(&mut self.cached, value);
        unsafe { Storage::set_byte(self.slot, self.offset.into(), value as u8) }
    }
}

impl<'b, H: Host> StorageType<'b, H> for StorageBool<'b, H> {
    type Wraps<'a>
        = bool
    where
        Self: 'a;
    type WrapsMut<'a>
        = StorageGuardMut<'a, Self>
    where
        Self: 'a;

    const SLOT_BYTES: usize = 1;

    unsafe fn new(slot: U256, offset: u8, host: &'b H) -> Self {
        Self {
            slot,
            offset,
            cached: OnceCell::new(),
            host,
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

impl<'b, H: Host> SimpleStorageType<'b, H> for StorageBool<'b, H> {
    fn set_by_wrapped(&mut self, value: Self::Wraps<'b>) {
        self.set(value);
    }
}

impl<'a, H: Host> Erase<'a, H> for StorageBool<'a, H> {
    fn erase(&mut self) {
        self.set(false);
    }
}

impl<'a, H: Host> Deref for StorageBool<'a, H> {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        self.cached.get_or_init(|| unsafe {
            let data = Storage::get_byte(self.slot, self.offset.into());
            data != 0
        })
    }
}

impl<'a, H: Host> From<StorageBool<'a, H>> for bool {
    fn from(value: StorageBool<'a, H>) -> Self {
        *value
    }
}

/// Accessor for a storage-backed [`Address`].
#[derive(Debug)]
pub struct StorageAddress<'a, H: Host> {
    slot: U256,
    offset: u8,
    cached: OnceCell<Address>,
    host: &'a H,
}

impl<'a, H: Host> StorageAddress<'a, H> {
    /// Gets the underlying [`Address`] in persistent storage.
    pub fn get(&self) -> Address {
        **self
    }

    /// Gets the underlying [`Address`] in persistent storage.
    pub fn set(&mut self, value: Address) {
        overwrite_cell(&mut self.cached, value);
        unsafe { Storage::set::<20>(self.slot, self.offset.into(), value.into()) }
    }
}

impl<'b, H: Host> StorageType<'b, H> for StorageAddress<'b, H> {
    type Wraps<'a>
        = Address
    where
        Self: 'a;
    type WrapsMut<'a>
        = StorageGuardMut<'a, Self>
    where
        Self: 'a;

    const SLOT_BYTES: usize = 20;

    unsafe fn new(slot: U256, offset: u8, host: &'b H) -> Self {
        Self {
            slot,
            offset,
            cached: OnceCell::new(),
            host,
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

impl<'a, H: Host> SimpleStorageType<'a, H> for StorageAddress<'a, H> {
    fn set_by_wrapped(&mut self, value: Self::Wraps<'a>) {
        self.set(value);
    }
}

impl<'a, H: Host> Erase<'a, H> for StorageAddress<'a, H> {
    fn erase(&mut self) {
        self.set(Self::Wraps::ZERO);
    }
}

impl<'a, H: Host> Deref for StorageAddress<'a, H> {
    type Target = Address;

    fn deref(&self) -> &Self::Target {
        self.cached
            .get_or_init(|| unsafe { Storage::get::<20>(self.slot, self.offset.into()).into() })
    }
}

impl<'a, H: Host> From<StorageAddress<'a, H>> for Address {
    fn from(value: StorageAddress<'a, H>) -> Self {
        *value
    }
}

/// Accessor for a storage-backed [`BlockNumber`].
///
/// This storage type allows convenient and type-safe storage of a
/// [`BlockNumber`].
#[derive(Debug)]
pub struct StorageBlockNumber<'a, H: Host> {
    slot: U256,
    offset: u8,
    cached: OnceCell<BlockNumber>,
    host: &'a H,
}

impl<'a, H: Host> StorageBlockNumber<'a, H> {
    /// Gets the underlying [`BlockNumber`] in persistent storage.
    pub fn get(&self) -> BlockNumber {
        **self
    }

    /// Sets the underlying [`BlockNumber`] in persistent storage.
    pub fn set(&mut self, value: BlockNumber) {
        overwrite_cell(&mut self.cached, value);
        let value = FixedBytes::from(value.to_be_bytes());
        unsafe { Storage::set::<8>(self.slot, self.offset.into(), value) };
    }
}

impl<'b, H: Host> StorageType<'b, H> for StorageBlockNumber<'b, H> {
    type Wraps<'a>
        = BlockNumber
    where
        Self: 'a;
    type WrapsMut<'a>
        = StorageGuardMut<'a, Self>
    where
        Self: 'a;

    const SLOT_BYTES: usize = 8;

    unsafe fn new(slot: U256, offset: u8, host: &'b H) -> Self {
        Self {
            slot,
            offset,
            cached: OnceCell::new(),
            host,
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

impl<'a, H: Host> SimpleStorageType<'a, H> for StorageBlockNumber<'a, H> {
    fn set_by_wrapped(&mut self, value: Self::Wraps<'a>) {
        self.set(value);
    }
}

impl<'a, H: Host> Erase<'a, H> for StorageBlockNumber<'a, H> {
    fn erase(&mut self) {
        self.set(0);
    }
}

impl<'a, H: Host> Deref for StorageBlockNumber<'a, H> {
    type Target = BlockNumber;

    fn deref(&self) -> &Self::Target {
        self.cached.get_or_init(|| unsafe {
            let data = Storage::get::<8>(self.slot, self.offset.into());
            u64::from_be_bytes(data.0)
        })
    }
}

impl<'a, H: Host> From<StorageBlockNumber<'a, H>> for BlockNumber {
    fn from(value: StorageBlockNumber<'a, H>) -> Self {
        *value
    }
}

/// Accessor for a storage-backed [`BlockHash`].
///
/// This storage type allows convenient and type-safe storage of a
/// [`BlockHash`].
#[derive(Clone, Debug)]
pub struct StorageBlockHash<'a, H: Host> {
    slot: U256,
    cached: OnceCell<BlockHash>,
    host: &'a H,
}

impl<'a, H: Host> StorageBlockHash<'a, H> {
    /// Gets the underlying [`BlockHash`] in persistent storage.
    pub fn get(&self) -> BlockHash {
        **self
    }

    /// Sets the underlying [`BlockHash`] in persistent storage.
    pub fn set(&mut self, value: BlockHash) {
        overwrite_cell(&mut self.cached, value);
        unsafe { Storage::set_word(self.slot, value) }
    }
}

impl<'b, H: Host> StorageType<'b, H> for StorageBlockHash<'b, H> {
    type Wraps<'a>
        = BlockHash
    where
        Self: 'a;
    type WrapsMut<'a>
        = StorageGuardMut<'a, Self>
    where
        Self: 'a;

    unsafe fn new(slot: U256, _offset: u8, host: &'b H) -> Self {
        let cached = OnceCell::new();
        Self { slot, cached, host }
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

impl<'a, H: Host> SimpleStorageType<'a, H> for StorageBlockHash<'a, H> {
    fn set_by_wrapped(&mut self, value: Self::Wraps<'a>) {
        self.set(value);
    }
}

impl<'a, H: Host> Erase<'a, H> for StorageBlockHash<'a, H> {
    fn erase(&mut self) {
        self.set(Self::Wraps::ZERO);
    }
}

impl<'a, H: Host> Deref for StorageBlockHash<'a, H> {
    type Target = BlockHash;

    fn deref(&self) -> &Self::Target {
        self.cached.get_or_init(|| Storage::get_word(self.slot))
    }
}

impl<'a, H: Host> From<StorageBlockHash<'a, H>> for BlockHash {
    fn from(value: StorageBlockHash<'a, H>) -> Self {
        *value
    }
}

/// We implement `StorageType` for `PhantomData` so that storage types can be generic.
impl<'b, T, H: Host> StorageType<'b, H> for PhantomData<T> {
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

    unsafe fn new(_slot: U256, _offset: u8, _host: &'b H) -> Self {
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
