// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use crate::ArbResult;
use alloy_primitives::{Address, B256, U256};
use stylus_core::Host;

/// Mechanism for performing raw calls to other contracts.
///
/// For safe calls, see [`Call`](super::Call).
#[derive(Clone)]
#[must_use]
pub struct RawCall<'a, H: Host + ?Sized> {
    kind: CallKind,
    callvalue: U256,
    gas: Option<u64>,
    offset: usize,
    size: Option<usize>,
    #[allow(unused)]
    cache_policy: CachePolicy,
    host: &'a H,
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

impl<'a, H: Host + ?Sized> RawCall<'a, H> {
    /// Begin configuring the raw call, similar to how [`std::fs::OpenOptions`][OpenOptions] works.
    ///
    /// ```no_run
    /// use stylus_sdk::call::RawCall;
    /// use stylus_sdk::stylus_core::host::Host;
    /// use stylus_sdk::{alloy_primitives::address, hex};
    /// use stylus_sdk::host::WasmVM;
    ///
    /// fn do_call(host: &impl Host) -> Result<(), ()> {
    ///     let contract = address!("361594F5429D23ECE0A88E4fBE529E1c49D524d8");
    ///     let calldata = &hex::decode("eddecf107b5740cef7f5a01e3ea7e287665c4e75").unwrap();
    ///
    ///     unsafe {
    ///         let result = RawCall::new(host)       // configure a call
    ///             .gas(2100)                    // supply 2100 gas
    ///             .limit_return_data(0, 32)     // only read the first 32 bytes back
    ///             .flush_storage_cache()        // flush the storage cache before the call
    ///             .call(contract, calldata);    // do the call
    ///     }
    ///     Ok(())
    /// }
    /// ```
    ///
    /// [OpenOptions]: https://doc.rust-lang.org/stable/std/fs/struct.OpenOptions.html
    pub fn new(host: &'a H) -> Self {
        Self {
            host,
            cache_policy: CachePolicy::default(),
            kind: CallKind::default(),
            callvalue: U256::ZERO,
            gas: None,
            offset: 0,
            size: None,
        }
    }

    /// Configures a call that supplies callvalue, denominated in wei.
    pub fn new_with_value(host: &'a H, callvalue: U256) -> Self {
        Self {
            host,
            callvalue,
            cache_policy: CachePolicy::default(),
            kind: CallKind::default(),
            gas: None,
            offset: 0,
            size: None,
        }
    }

    /// Begin configuring a [`delegate call`].
    ///
    /// [`DELEGATE_CALL`]: https://www.evm.codes/#F4
    pub fn new_delegate(host: &'a H) -> Self {
        Self {
            host,
            cache_policy: CachePolicy::default(),
            kind: CallKind::Delegate,
            callvalue: U256::ZERO,
            gas: None,
            offset: 0,
            size: None,
        }
    }

    /// Begin configuring a [`static call`].
    ///
    /// [`STATIC_CALL`]: https://www.evm.codes/#FA
    pub fn new_static(host: &'a H) -> Self {
        Self {
            host,
            cache_policy: CachePolicy::default(),
            kind: CallKind::Static,
            callvalue: U256::ZERO,
            gas: None,
            offset: 0,
            size: None,
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
    /// [`Ink and Gas`]: https://docs.arbitrum.io/stylus/concepts/gas-metering
    #[allow(dead_code)]
    pub fn ink(mut self, ink: u64) -> Self {
        self.gas = Some(self.host.ink_to_gas(ink));
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
    pub fn flush_storage_cache(mut self) -> Self {
        self.cache_policy = self.cache_policy.max(CachePolicy::Flush);
        self
    }

    /// Flush and clear the storage cache before the call.
    pub fn clear_storage_cache(mut self) -> Self {
        self.cache_policy = CachePolicy::Clear;
        self
    }

    /// Performs a raw call to another contract at the given address with the given `calldata`.
    ///
    /// # Safety
    ///
    /// This function is `unsafe`. That's because raw calls might alias storage if used in the
    /// middle of a storage ref's lifetime.
    ///
    /// For extra flexibility, this method does not clear the global storage cache by default.
    /// See [`flush_storage_cache`] and [`clear_storage_cache`] for more information.
    ///
    /// [`flush_storage_cache`]: RawCall::flush_storage_cache
    /// [`clear_storage_cache`]: RawCall::clear_storage_cache
    pub unsafe fn call(self, contract: Address, calldata: &[u8]) -> ArbResult {
        let mut outs_len: usize = 0;
        let gas = self.gas.unwrap_or(u64::MAX); // will be clamped by 63/64 rule
        let value = B256::from(self.callvalue);
        let status = unsafe {
            match self.cache_policy {
                CachePolicy::Clear => self.host.flush_cache(true),
                CachePolicy::Flush => self.host.flush_cache(false),
                CachePolicy::DoNothing => {}
            }
            match self.kind {
                CallKind::Basic => self.host.call_contract(
                    contract.as_ptr(),
                    calldata.as_ptr(),
                    calldata.len(),
                    value.as_ptr(),
                    gas,
                    &mut outs_len,
                ),
                CallKind::Delegate => self.host.delegate_call_contract(
                    contract.as_ptr(),
                    calldata.as_ptr(),
                    calldata.len(),
                    gas,
                    &mut outs_len,
                ),
                CallKind::Static => self.host.static_call_contract(
                    contract.as_ptr(),
                    calldata.as_ptr(),
                    calldata.len(),
                    gas,
                    &mut outs_len,
                ),
            }
        };

        let outs = self.host.read_return_data(self.offset, self.size);
        match status {
            0 => Ok(outs),
            _ => Err(outs),
        }
    }
}
