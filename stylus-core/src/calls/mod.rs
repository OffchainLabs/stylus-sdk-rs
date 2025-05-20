// Copyright 2025-2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy_primitives::U256;

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
pub struct Call<S, const HAS_VALUE: bool = false> {
    gas: u64,
    value: Option<U256>,
    storage: S,
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

impl<S> StaticCallContext for Call<S, false> {}

impl<S> NonPayableCallContext for Call<S, false> {}

unsafe impl<S, const HAS_VALUE: bool> MutatingCallContext for Call<S, HAS_VALUE> {
    fn value(&self) -> U256 {
        self.value.unwrap_or_default()
    }
}
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
