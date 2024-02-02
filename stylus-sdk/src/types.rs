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
use alloc::vec::Vec;
use alloy_primitives::{b256, Address, B256, U256};

/// Trait that allows the [`Address`] type to inspect the corresponding account's balance and codehash.
pub trait AddressVM {
    /// The balance in wei of the account.
    fn balance(&self) -> U256;

    /// The account's code.
    ///
    /// Returns an empty [`vec`] for [`EOAs`].
    ///
    /// [`EOAs`]: https://ethereum.org/en/developers/docs/accounts/#types-of-account
    fn code(&self) -> Vec<u8>;

    /// The length of the account's code in bytes.
    ///
    /// Returns `0` for [`EOAs`].
    ///
    /// [`EOAs`]: https://ethereum.org/en/developers/docs/accounts/#types-of-account
    fn code_size(&self) -> usize;

    /// The codehash of the contract or [`EOA`] at the given address.
    ///
    /// [`EOA`]: https://ethereum.org/en/developers/docs/accounts/#types-of-account
    fn codehash(&self) -> B256;

    /// Determines if an account has code. Note that this is insufficient to determine if an address is an
    /// [`EOA`]. During contract deployment, an account only gets its code at the very end, meaning that
    /// this method will return `false` while the constructor is executing.
    ///
    /// [`EOA`]: https://ethereum.org/en/developers/docs/accounts/#types-of-account
    fn has_code(&self) -> bool;

    fn expmod(&self, x: U256, y: U256, z: U256) -> U256 {
        let mut result = U256::from(0);
        unsafe { hostio::expmod(
            &x as *const U256 as *const u8,
            &y as *const U256 as *const u8,
            &z as *const U256 as *const u8,
            &mut result as *mut U256 as *mut u8,
        )};
        result
    }
}

impl AddressVM for Address {
    fn balance(&self) -> U256 {
        let mut data = [0; 32];
        unsafe { hostio::account_balance(self.as_ptr(), data.as_mut_ptr()) };
        U256::from_be_bytes(data)
    }

    fn code(&self) -> Vec<u8> {
        let size = self.code_size();
        let mut data = Vec::with_capacity(size);
        unsafe {
            hostio::account_code(self.as_ptr(), 0, size, data.as_mut_ptr());
            data.set_len(size);
        }
        data
    }

    fn code_size(&self) -> usize {
        unsafe { hostio::account_code_size(self.as_ptr()) }
    }

    fn codehash(&self) -> B256 {
        let mut data = [0; 32];
        unsafe { hostio::account_codehash(self.as_ptr(), data.as_mut_ptr()) };
        data.into()
    }

    fn has_code(&self) -> bool {
        let hash = self.codehash();
        !hash.is_zero()
            && hash != b256!("c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470")
    }
}
