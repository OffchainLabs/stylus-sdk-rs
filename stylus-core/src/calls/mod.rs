// Copyright 2025-2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy_primitives::U256;

use crate::TopLevelStorage;

pub mod errors;

/// Trait for calling other contracts.
/// Users should rarely implement this trait outside of proc macros.
pub trait CallContext {
    /// Amount of gas to supply the call.
    /// Note: values are clipped to the amount of gas remaining.
    fn gas(&self) -> u64;
}

/// Trait for calling the `view` or `pure` methods of other contracts.
/// Users should rarely implement this trait outside of proc macros.
pub trait StaticCallContext: CallContext {}

/// Trait for calling the mutable methods of other contracts.
/// Users should rarely implement this trait outside of proc macros.
///
/// # Safety
///
/// The type must contain a [`TopLevelStorage`][TLS] to prevent aliasing in cases of reentrancy.
///
/// [TLS]: stylus_core::storage::TopLevelStorage
pub unsafe trait MutatingCallContext: CallContext {
    /// Amount of ETH in wei to give the other contract.
    fn value(&self) -> U256;
}

/// Trait for calling the `write` methods of other contracts.
/// Users should rarely implement this trait outside of proc macros.
///
/// Note: any implementations of this must return zero for [`MutatingCallContext::value`].
pub trait NonPayableCallContext: MutatingCallContext {}

/// Enables configurable calls to other contracts.
#[derive(Debug, Clone)]
pub struct Call<const MUTATING: bool = false, const HAS_VALUE: bool = false> {
    gas: u64,
    value: Option<U256>,
}

impl<const MUTATING: bool, const HAS_VALUE: bool> Call<MUTATING, HAS_VALUE> {
    /// Amount of gas to supply the call.
    /// Values greater than the amount provided will be clipped to all gas left.
    pub fn gas(self, gas: u64) -> Self {
        Self { gas, ..self }
    }

    /// Amount of ETH in wei to give the other contract.
    /// Note: adding value will prevent calls to non-payable methods.
    pub fn value(self, value: U256) -> Call<true, true> {
        Call {
            value: Some(value),
            gas: self.gas,
        }
    }
}

impl<const MUTATING: bool, const HAS_VALUE: bool> CallContext for Call<MUTATING, HAS_VALUE> {
    fn gas(&self) -> u64 {
        self.gas
    }
}

impl StaticCallContext for Call<false, false> {}

impl NonPayableCallContext for Call<true, false> {}

unsafe impl<const HAS_VALUE: bool> MutatingCallContext for Call<true, HAS_VALUE> {
    fn value(&self) -> U256 {
        self.value.unwrap_or_default()
    }
}

impl Default for Call<false, false> {
    fn default() -> Self {
        Self::new()
    }
}

impl Call<false, false> {
    pub fn new() -> Self {
        Self {
            gas: u64::MAX,
            value: None,
        }
    }
}

impl Call<true, false> {
    pub fn new_mutating(_storage: &mut impl TopLevelStorage) -> Self {
        Self {
            gas: 0,
            value: None,
        }
    }
}

impl Call<true, true> {
    pub fn new_payable(_storage: &mut impl TopLevelStorage, value: U256) -> Self {
        Self {
            gas: 0,
            value: Some(value),
        }
    }
}
