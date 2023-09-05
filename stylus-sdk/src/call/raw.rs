// Copyright 2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use crate::{
    contract::{read_return_data, RETURN_DATA_LEN},
    hostio, tx, ArbResult,
};
use alloy_primitives::{Address, B256, U256};

#[cfg(feature = "storage-cache")]
use crate::storage::StorageCache;

/// Mechanism for performing raw calls to other contracts.
#[derive(Clone, Default)]
#[must_use]
pub struct RawCall {
    kind: CallKind,
    callvalue: U256,
    gas: Option<u64>,
    offset: usize,
    size: Option<usize>,
    #[allow(unused)]
    cache_policy: CachePolicy,
}

/// What kind of call to perform.
#[derive(Clone, Default, PartialEq)]
enum CallKind {
    #[default]
    Basic,
    Delegate,
    Static,
}

/// How to manage the storage cache, if enabled.
#[allow(unused)]
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CachePolicy {
    #[default]
    DoNothing,
    Flush,
    Clear,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct RustVec {
    ptr: *mut u8,
    len: usize,
    cap: usize,
}

impl Default for RustVec {
    fn default() -> Self {
        Self {
            ptr: core::ptr::null_mut(),
            len: 0,
            cap: 0,
        }
    }
}

impl RawCall {
    /// Begin configuring the raw call.
    pub fn new() -> Self {
        Default::default()
    }

    /// Configures a call that supplies callvalue, denominated in wei.
    pub fn new_with_value(callvalue: U256) -> Self {
        Self {
            callvalue,
            ..Default::default()
        }
    }

    /// Begin configuring a [`delegate call`].
    ///
    /// [`DELEGATE_CALL`]: https://www.evm.codes/#F4
    pub fn new_delegate() -> Self {
        Self {
            kind: CallKind::Delegate,
            ..Default::default()
        }
    }

    /// Begin configuring a [`static call`].
    ///
    /// [`STATIC_CALL`]: https://www.evm.codes/#FA
    pub fn new_static() -> Self {
        Self {
            kind: CallKind::Static,
            ..Default::default()
        }
    }

    /// Configures the amount of gas to supply.
    /// Note: large values are clipped to the amount of gas remaining.
    pub fn gas(mut self, gas: u64) -> Self {
        self.gas = Some(gas);
        self
    }

    /// Configures the amount of ink to supply.
    /// Note: values are clipped to the amount of ink remaining.
    /// See [`Ink and Gas`] for more information on Stylus's compute-pricing model.
    ///
    /// [`Ink and Gas`]: https://developer.arbitrum.io/TODO
    pub fn ink(mut self, ink: u64) -> Self {
        self.gas = Some(tx::ink_to_gas(ink));
        self
    }

    /// Configures what portion of the return data to copy.
    /// Does not revert if out of bounds, but rather copies the overlapping portion.
    pub fn limit_return_data(mut self, offset: usize, size: usize) -> Self {
        self.offset = offset;
        self.size = Some(size);
        self
    }

    /// Configures the call to avoid copying any return data.
    /// Equivalent to `limit_return_data(0, 0)`.
    pub fn skip_return_data(self) -> Self {
        self.limit_return_data(0, 0)
    }

    /// Write all cached values to persistent storage before the call.
    #[cfg(feature = "storage-cache")]
    pub fn flush_storage_cache(mut self) -> Self {
        self.cache_policy = self.cache_policy.max(CachePolicy::Flush);
        self
    }

    /// Flush and clear the storage cache before the call.
    #[cfg(feature = "storage-cache")]
    pub fn clear_storage_cache(mut self) -> Self {
        self.cache_policy = CachePolicy::Clear;
        self
    }

    /// Performs a raw call to another contract at the given address with the given `calldata`.
    ///
    /// # Safety
    ///
    /// Enables storage aliasing if used in the middle of a storage reference's lifetime and reentrancy is allowed.
    ///
    /// For extra flexibility, this method does not clear the global storage cache.
    /// See [`StorageCache::flush`] and [`StorageCache::clear`] for more information.
    pub unsafe fn call(self, contract: Address, calldata: &[u8]) -> ArbResult {
        let mut outs_len = 0;
        let gas = self.gas.unwrap_or(u64::MAX); // will be clamped by 63/64 rule
        let value = B256::from(self.callvalue);
        let status = unsafe {
            #[cfg(feature = "storage-cache")]
            match self.cache_policy {
                CachePolicy::Clear => StorageCache::clear(),
                CachePolicy::Flush => StorageCache::flush(),
                CachePolicy::DoNothing => {}
            }
            match self.kind {
                CallKind::Basic => hostio::call_contract(
                    contract.as_ptr(),
                    calldata.as_ptr(),
                    calldata.len(),
                    value.as_ptr(),
                    gas,
                    &mut outs_len,
                ),
                CallKind::Delegate => hostio::delegate_call_contract(
                    contract.as_ptr(),
                    calldata.as_ptr(),
                    calldata.len(),
                    gas,
                    &mut outs_len,
                ),
                CallKind::Static => hostio::static_call_contract(
                    contract.as_ptr(),
                    calldata.as_ptr(),
                    calldata.len(),
                    gas,
                    &mut outs_len,
                ),
            }
        };

        unsafe {
            RETURN_DATA_LEN.set(outs_len);
        }

        let outs = read_return_data(self.offset, self.size);
        match status {
            0 => Ok(outs),
            _ => Err(outs),
        }
    }
}
