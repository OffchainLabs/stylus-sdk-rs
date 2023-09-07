// Copyright 2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

//! Traits for common types.
//!
//! The contents of this module are typically imported via the [`prelude`](crate::prelude).
//!
//! ```no_run
//! use stylus_sdk::prelude::*;
//! use alloy_primitives::{address, Address};
//!
//! let account = address!("361594F5429D23ECE0A88E4fBE529E1c49D524d8");
//! let balance = account.balance();
//! ```

use crate::hostio;
use alloy_primitives::{Address, B256, U256};

/// Trait that allows the [`Address`] type to inspect the corresponding account's balance and codehash.
pub trait AddressVM {
    /// The balance in wei of the account.
    fn balance(&self) -> U256;

    /// The codehash of the contract at the given address, or `None` when an [`EOA`].
    ///
    /// [`EOA`]: https://ethereum.org/en/developers/docs/accounts/#types-of-account
    fn codehash(&self) -> Option<B256>;
}

impl AddressVM for Address {
    fn balance(&self) -> U256 {
        let mut data = [0; 32];
        unsafe { hostio::account_balance(self.0.as_ptr(), data.as_mut_ptr()) };
        U256::from_be_bytes(data)
    }

    fn codehash(&self) -> Option<B256> {
        let mut data = [0; 32];
        unsafe { hostio::account_codehash(self.0.as_ptr(), data.as_mut_ptr()) };
        (data != [0; 32]).then_some(data.into())
    }
}
