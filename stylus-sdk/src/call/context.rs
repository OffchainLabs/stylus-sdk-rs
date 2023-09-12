// Copyright 2022-2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use crate::storage::TopLevelStorage;

use super::{CallContext, MutatingCallContext, NonPayableCallContext, StaticCallContext};
use alloy_primitives::U256;
use cfg_if::cfg_if;

/// Enables configurable calls to other contracts.
#[derive(Debug, Clone)]
pub struct Call<S, const HAS_VALUE: bool = false> {
    gas: u64,
    value: Option<U256>,
    storage: S,
}

impl<'a, S: TopLevelStorage> Call<&'a mut S, false>
where
    S: TopLevelStorage + 'a,
{
    /// Similar to [`new`], but intended for projects and libraries using reentrant patterns.
    ///
    /// [`new_in`] safeguards persistent storage by requiring a reference to a [`TopLevelStorage`] `struct`.
    ///
    /// Recall that [`TopLevelStorage`] is special in that a reference to it represents access to the entire
    /// contract's state. So that it's sound to [`flush`] or [`clear`] the [`StorageCache`] when calling out
    /// to other contracts, calls that may induce reentrancy require an `&` or `&mut` to one.
    ///
    /// ```no_run
    /// use stylus_sdk::call::{Call, Error};
    /// use stylus_sdk::{prelude::*, evm, msg, alloy_primitives::Address};
    /// extern crate alloc;
    ///
    /// sol_interface! {
    ///     interface IService {
    ///         function makePayment(address user) payable returns (string);
    ///     }
    /// }
    ///
    /// pub fn do_call(
    ///     storage: &mut impl TopLevelStorage,  // can be generic, but often just &mut self
    ///     account: IService,                   // serializes as an Address
    ///     user: Address,
    /// ) -> Result<String, Error> {
    ///
    ///     let config = Call::new_in(storage)
    ///         .gas(evm::gas_left() / 2)        // limit to half the gas left
    ///         .value(msg::value());            // set the callvalue
    ///
    ///     account.make_payment(config, user)   // note the snake case
    /// }
    /// ```
    ///
    /// Projects that opt out of the [`StorageCache`] by disabling the `storage-cache` feature
    /// may ignore this method.
    ///
    /// [`StorageCache`]: crate::storage::StorageCache
    /// [`flush`]: crate::storage::StorageCache::flush
    /// [`clear`]: crate::storage::StorageCache::clear
    /// [`new_in`]: Call::new_in
    /// [`new`]: Call::new
    pub fn new_in(storage: &'a mut S) -> Self {
        Self {
            gas: u64::MAX,
            value: None,
            storage,
        }
    }
}

impl Default for Call<(), false> {
    fn default() -> Self {
        Self::new()
    }
}

impl Call<(), false> {
    /// Begin configuring a call, similar to how [`RawCall`](super::RawCall) and [`std::fs::OpenOptions`] work.
    ///
    /// ```ignore
    /// use stylus_sdk::call::{Call, Error};
    /// use stylus_sdk::{prelude::*, evm, msg, alloy_primitives::Address};
    /// extern crate alloc;
    ///
    /// sol_interface! {
    ///     interface IService {
    ///         function makePayment(address user) payable returns (string);
    ///     }
    /// }
    ///
    /// pub fn do_call(account: IService, user: Address) -> Result<String, Error> {
    ///     let config = Call::new()
    ///         .gas(evm::gas_left() / 2)       // limit to half the gas left
    ///         .value(msg::value());           // set the callvalue
    ///
    ///     account.make_payment(config, user)  // note the snake case
    /// }
    /// ```
    pub fn new() -> Self {
        Self {
            gas: u64::MAX,
            value: None,
            storage: (),
        }
    }
}

impl<S, const HAS_VALUE: bool> Call<S, HAS_VALUE> {
    /// Amount of gas to supply the call.
    /// Values greater than the amount provided will be clipped to all gas left.
    pub fn gas(self, gas: u64) -> Self {
        Self { gas, ..self }
    }

    /// Amount of ETH in wei to give the other contract.
    /// Note: adding value will prevent calls to non-payable methods.
    pub fn value(self, value: U256) -> Call<S, true> {
        Call {
            value: Some(value),
            gas: self.gas,
            storage: self.storage,
        }
    }
}

impl<S, const HAS_VALUE: bool> CallContext for Call<S, HAS_VALUE> {
    fn gas(&self) -> u64 {
        self.gas
    }
}

// allow &self as a context
impl<'a, T> CallContext for &'a T
where
    T: TopLevelStorage,
{
    fn gas(&self) -> u64 {
        u64::MAX
    }
}

// allow &mut self as a context
impl<T> CallContext for &mut T
where
    T: TopLevelStorage,
{
    fn gas(&self) -> u64 {
        u64::MAX
    }
}

// allow &self to be a `pure` and `static` call context
impl<'a, T> StaticCallContext for &'a T where T: TopLevelStorage {}

// allow &mut self to be a `pure` and `static` call context
impl<'a, T> StaticCallContext for &'a mut T where T: TopLevelStorage {}

// allow &mut self to be a `write` and `payable` call context
unsafe impl<T> MutatingCallContext for &mut T
where
    T: TopLevelStorage,
{
    fn value(&self) -> U256 {
        U256::ZERO
    }
}

// allow &mut self to be a `write`-only call context
impl<T> NonPayableCallContext for &mut T where T: TopLevelStorage {}

cfg_if! {
    if #[cfg(all(feature = "storage-cache", feature = "reentrant"))] {
        // The following impls safeguard state during reentrancy scenarios

        impl<S: TopLevelStorage> StaticCallContext for Call<&S, false> {}

        impl<S: TopLevelStorage> StaticCallContext for Call<&mut S, false> {}

        impl<S: TopLevelStorage> NonPayableCallContext for Call<&mut S, false> {}

        unsafe impl<S: TopLevelStorage, const HAS_VALUE: bool> MutatingCallContext
            for Call<&mut S, HAS_VALUE>
        {
            fn value(&self) -> U256 {
                self.value.unwrap_or_default()
            }
        }
    } else {
        // If there's no reentrancy, all calls are storage safe

        impl<S> StaticCallContext for Call<S, false> {}

        impl<S> NonPayableCallContext for Call<S, false> {}

        unsafe impl<S, const HAS_VALUE: bool> MutatingCallContext for Call<S, HAS_VALUE> {
            fn value(&self) -> U256 {
                self.value.unwrap_or_default()
            }
        }
    }
}
