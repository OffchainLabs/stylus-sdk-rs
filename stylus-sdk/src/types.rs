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

    /// Gets the code at the given address. The semantics are equivalent to that of the EVM's [`EXT_CODESIZE`].
    ///
    /// [`EXT_CODE_COPY`]: https://www.evm.codes/#3C
    fn code(&self) -> Vec<u8>;

    /// Gets the size of the code in bytes at the given address. The semantics are equivalent
    /// to that of the EVM's [`EXT_CODESIZE`].
    ///
    /// [`EXT_CODESIZE`]: https://www.evm.codes/#3B
    fn code_size(&self) -> usize;

    /// The codehash of the contract or [`EOA`] at the given address.
    ///
    /// [`EOA`]: https://ethereum.org/en/developers/docs/accounts/#types-of-account
    fn code_hash(&self) -> B256;

    /// Determines if an account has code. Note that this is insufficient to determine if an address is an
    /// [`EOA`]. During contract deployment, an account only gets its code at the very end, meaning that
    /// this method will return `false` while the constructor is executing.
    ///
    /// [`EOA`]: https://ethereum.org/en/developers/docs/accounts/#types-of-account
    fn has_code(&self) -> bool;
}

impl AddressVM for Address {
    fn balance(&self) -> U256 {
        let mut data = [0; 32];
        unsafe { hostio::account_balance(self.as_ptr(), data.as_mut_ptr()) };
        U256::from_be_bytes(data)
    }

    fn code(&self) -> Vec<u8> {
        let size = self.code_size();
        let mut dest = Vec::with_capacity(size);
        unsafe {
            hostio::account_code(self.as_ptr(), 0, size, dest.as_mut_ptr());
            dest.set_len(size);
            dest
        }
    }

    fn code_size(&self) -> usize {
        unsafe { hostio::account_code_size(self.as_ptr()) }
    }

    fn code_hash(&self) -> B256 {
        let mut data = [0; 32];
        unsafe { hostio::account_codehash(self.as_ptr(), data.as_mut_ptr()) };
        data.into()
    }

    fn has_code(&self) -> bool {
        let hash = self.code_hash();
        !hash.is_zero()
            && hash != b256!("c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470")
    }
}
