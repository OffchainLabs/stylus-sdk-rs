// Copyright 2022-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Call other contracts.
//!
//! There are two primary ways to make calls to other contracts via the Stylus SDK.
//! - [`Call`] with [`sol_interface!`][sol_interface] for richly-typed calls.
//! - [`RawCall`] for `unsafe`, bytes-in bytes-out calls.
//!
//! Additional helpers exist for specific use-cases like [`transfer_eth`].
//!
//! [sol_interface]: crate::prelude::sol_interface

use alloc::vec::Vec;
use alloy_primitives::Address;

pub(crate) use raw::CachePolicy;
pub use raw::RawCall;
use stylus_core::{
    calls::{errors::Error, MutatingCallContext, StaticCallContext},
    Host,
};

mod raw;

/// Provides a convenience method to transfer ETH to a given address.
pub mod transfer;

/// Static calls the contract at the given address.
pub fn static_call(
    host: &dyn Host,
    context: impl StaticCallContext,
    to: Address,
    data: &[u8],
) -> Result<Vec<u8>, Error> {
    host.flush_cache(false); // flush storage to persist changes, but don't invalidate the cache
    unsafe {
        RawCall::new_static(host)
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
pub unsafe fn delegate_call(
    host: &dyn Host,
    context: impl MutatingCallContext,
    to: Address,
    data: &[u8],
) -> Result<Vec<u8>, Error> {
    host.flush_cache(true); // clear storage to persist changes, invalidating the cache

    RawCall::new_delegate(host)
        .gas(context.gas())
        .call(to, data)
        .map_err(Error::Revert)
}

/// Calls the contract at the given address.
pub fn call(
    host: &dyn Host,
    context: impl MutatingCallContext,
    to: Address,
    data: &[u8],
) -> Result<Vec<u8>, Error> {
    host.flush_cache(true); // clear storage to persist changes, invalidating the cache

    unsafe {
        RawCall::new_with_value(host, context.value())
            .gas(context.gas())
            .call(to, data)
            .map_err(Error::Revert)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alloy_primitives::{Address, U256};
    use stylus_core::CallContext;
    use stylus_test::TestVM;

    #[derive(Clone)]
    pub struct MyContract;
    impl CallContext for MyContract {
        fn gas(&self) -> u64 {
            0
        }
    }
    unsafe impl MutatingCallContext for MyContract {
        fn value(&self) -> U256 {
            U256::from(0)
        }
    }
    impl StaticCallContext for MyContract {}

    #[test]
    fn test_calls() {
        let vm = TestVM::new();
        let contract = MyContract {};
        let target = Address::from([2u8; 20]);
        let data = vec![1, 2, 3, 4];
        let expected_return = vec![5, 6, 7, 8];

        // Mock a regular call.
        vm.mock_call(
            target,
            data.clone(),
            U256::ZERO,
            Ok(expected_return.clone()),
        );

        let response = call(&vm, contract.clone(), target, &data).unwrap();
        assert_eq!(response, expected_return);
        vm.clear_mocks();

        // Mock a delegate call.
        vm.mock_delegate_call(target, data.clone(), Ok(expected_return.clone()));
        let response = unsafe { delegate_call(&vm, contract.clone(), target, &data).unwrap() };
        assert_eq!(response, expected_return);
        vm.clear_mocks();

        // Mock a static call.
        vm.mock_static_call(target, data.clone(), Ok(expected_return.clone()));
        let response = static_call(&vm, contract.clone(), target, &data).unwrap();
        assert_eq!(response, expected_return);
    }
}
