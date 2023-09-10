// Copyright 2022-2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use super::{CallContext, MutatingCallContext, NonPayableCallContext, StaticCallContext};
use crate::storage::TopLevelStorage;
use alloy_primitives::U256;

/// Enables configurable calls to other contracts.
#[derive(Debug, Clone)]
pub struct Context<S, const HAS_VALUE: bool = false> {
    gas: u64,
    value: U256,
    _storage: S,
}

impl Context<(), false> {
    /// Begin configuring a call, similar to how [`RawCall`](super::RawCall) and [`std::fs::OpenOptions`] work.
    ///
    /// ```no_run
    /// use stylus_sdk::call::{Context, Error};
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
    ///     account: IService,
    ///     user: Address,
    /// ) -> Result<String, Error> {
    ///
    ///     let context = Context::new()
    ///         .mutate(storage)             // make the call mutable (usually one passes self)
    ///         .gas(evm::gas_left() / 2)    // limit to half the gas left
    ///         .value(msg::value());        // set the callvalue
    ///
    ///     account.make_payment(context, user)  // note the snake case
    /// }
    /// ```
    pub fn new() -> Self {
        Self {
            gas: u64::MAX,
            value: U256::ZERO,
            _storage: (),
        }
    }
}

impl Default for Context<(), false> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, const HAS_VALUE: bool> Context<S, HAS_VALUE> {
    /// Assigns a [`TopLevelStorage`] so that mutable methods can be called.
    /// Note: enabling mutation will prevent calls to `pure` and `view` methods.
    pub fn mutate<NewS: TopLevelStorage>(
        self,
        storage: &mut NewS,
    ) -> Context<&mut NewS, HAS_VALUE> {
        Context {
            gas: self.gas,
            value: self.value,
            _storage: storage,
        }
    }

    /// Amount of gas to supply the call.
    /// Values greater than the amount provided will be clipped to all gas left.
    pub fn gas(self, gas: u64) -> Self {
        Self { gas, ..self }
    }

    /// Amount of ETH in wei to give the other contract.
    /// Note: adding value will prevent calls to non-payable methods.
    pub fn value(self, value: U256) -> Context<S, true> {
        Context {
            value,
            gas: self.gas,
            _storage: self._storage,
        }
    }
}

impl<S, const HAS_VALUE: bool> CallContext for Context<S, HAS_VALUE> {
    fn gas(&self) -> u64 {
        self.gas
    }
}

impl StaticCallContext for Context<(), false> {}

unsafe impl<S: TopLevelStorage, const HAS_VALUE: bool> MutatingCallContext
    for Context<&mut S, HAS_VALUE>
{
    fn value(&self) -> U256 {
        self.value
    }
}

impl<S: TopLevelStorage> NonPayableCallContext for Context<&mut S, false> {}

// allow &self to be a `pure` and `static` call context
impl<'a, T> CallContext for &'a T
where
    T: TopLevelStorage,
{
    fn gas(&self) -> u64 {
        u64::MAX
    }
}

impl<'a, T> StaticCallContext for &'a T where T: TopLevelStorage {}

// allow &mut self to be a non-static call context
impl<T> CallContext for &mut T
where
    T: TopLevelStorage,
{
    fn gas(&self) -> u64 {
        u64::MAX
    }
}

unsafe impl<T> MutatingCallContext for &mut T
where
    T: TopLevelStorage,
{
    fn value(&self) -> U256 {
        U256::ZERO
    }
}

impl<T> NonPayableCallContext for &mut T where T: TopLevelStorage {}
