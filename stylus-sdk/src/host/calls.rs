// Copyright 2025-2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloc::vec::Vec;
use alloy_primitives::{Address, U256};
use stylus_core::{
    calls::{errors::Error, CallAccess, MutatingCallContext, StaticCallContext, ValueTransfer},
    host::StorageAccess,
};

use crate::call::RawCall;

use super::WasmVM;

impl CallAccess for WasmVM {
    /// Calls the contract at the given address.
    fn call(
        &self,
        context: &dyn MutatingCallContext,
        to: alloy_primitives::Address,
        data: &[u8],
    ) -> Result<Vec<u8>, Error> {
        self.flush_cache(true); // clear the storage to persist changes, invalidating the cache
        unsafe {
            RawCall::new_with_value(context.value())
                .gas(context.gas())
                .call(to, data)
                .map_err(Error::Revert)
        }
    }
    /// Delegate calls the contract at the given address.
    ///
    /// # Safety
    ///
    /// A delegate call must trust the other contract to uphold safety requirements.
    /// Though this function clears any cached values, the other contract may arbitrarily change storage,
    /// spend ether, and do other things one should never blindly allow other contracts to do.
    unsafe fn delegate_call(
        &self,
        context: &dyn MutatingCallContext,
        to: alloy_primitives::Address,
        data: &[u8],
    ) -> Result<Vec<u8>, Error> {
        self.flush_cache(true); // clear the storage to persist changes, invalidating the cache
        unsafe {
            RawCall::new_delegate()
                .gas(context.gas())
                .call(to, data)
                .map_err(Error::Revert)
        }
    }
    /// Static calls the contract at the given address.
    fn static_call(
        &self,
        context: &dyn StaticCallContext,
        to: alloy_primitives::Address,
        data: &[u8],
    ) -> Result<Vec<u8>, Error> {
        self.flush_cache(false); // flush storage to persist changes, but don't invalidate the cache
        unsafe {
            RawCall::new_static()
                .gas(context.gas())
                .call(to, data)
                .map_err(Error::Revert)
        }
    }
}

impl ValueTransfer for WasmVM {
    /// Transfers an amount of ETH in wei to the given account.
    /// Note that this method will call the other contract, which may in turn call others.
    ///
    /// All gas is supplied, which the recipient may burn.
    /// If this is not desired, the [`call`] function may be used directly.
    ///
    /// [`call`]: super::call
    #[cfg(feature = "reentrant")]
    fn transfer_eth(
        &self,
        _storage: &mut dyn stylus_core::storage::TopLevelStorage,
        to: Address,
        amount: U256,
    ) -> Result<(), Vec<u8>> {
        self.flush_cache(true); // clear the storage to persist changes, invalidating the cache
        unsafe {
            RawCall::new_with_value(amount)
                .skip_return_data()
                .call(to, &[])?;
        }
        Ok(())
    }
    /// Transfers an amount of ETH in wei to the given account.
    /// Note that this method will call the other contract, which may in turn call others.
    ///
    /// All gas is supplied, which the recipient may burn.
    /// If this is not desired, the [`call`] function may be used directly.
    ///
    /// ```
    /// # use stylus_sdk::stylus_core::calls::{ValueTransfer, context::Call};
    /// # use stylus_test::*;
    /// # fn wrap() -> Result<(), Vec<u8>> {
    /// #   let vm = TestVM::default();
    /// #   let value = alloy_primitives::U256::ZERO;
    /// #   let recipient = alloy_primitives::Address::ZERO;
    /// vm.transfer_eth(recipient, value)?;                 // these two are equivalent
    /// vm.call(&Call::new().value(value), recipient, &[])?; // these two are equivalent
    /// #   Ok(())
    /// # }
    /// ```
    #[cfg(not(feature = "reentrant"))]
    fn transfer_eth(&self, to: Address, amount: U256) -> Result<(), Vec<u8>> {
        self.flush_cache(true); // clear the storage to persist changes, invalidating the cache
        unsafe {
            RawCall::new_with_value(amount)
                .skip_return_data()
                .call(to, &[])?;
        }
        Ok(())
    }
}
