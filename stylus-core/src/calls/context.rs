// Copyright 2025-2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use crate::{
    calls::{CallContext, MutatingCallContext, NonPayableCallContext, StaticCallContext},
    storage::TopLevelStorage,
};
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
    pub fn new_in(storage: &'a mut S) -> Self {
        Self {
            gas: u64::MAX,
            value: None,
            storage,
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
impl<T> CallContext for &T
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
impl<T> StaticCallContext for &T where T: TopLevelStorage {}

// allow &mut self to be a `pure` and `static` call context
impl<T> StaticCallContext for &mut T where T: TopLevelStorage {}

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
    if #[cfg(feature = "reentrant")] {
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

cfg_if! {
    if #[cfg(not(feature = "reentrant"))] {
        impl Default for Call<(), false> {
            fn default() -> Self {
                Self::new()
            }
        }

        impl Call<(), false> {
            pub fn new() -> Self {
                Self {
                    gas: u64::MAX,
                    value: None,
                    storage: (),
                }
            }
        }
    }
}
