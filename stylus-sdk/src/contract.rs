// Copyright 2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use crate::{
    hostio::{self, wrap_hostio},
    types::AddressVM,
};
use alloy_primitives::{Address, U256};

/// Reads the invocation's calldata.
/// The [`derive(Entrypoint)`] and [`entrypoint`] macros use this under the hood.
pub fn args(len: usize) -> Vec<u8> {
    let mut input = Vec::with_capacity(len);
    unsafe {
        hostio::read_args(input.as_mut_ptr());
        input.set_len(len);
    }
    input
}

/// Writes the contract's return data.
/// The [`derive(Entrypoint)`] and [`entrypoint!`] macros use this under the hood.
pub fn output(data: &[u8]) {
    unsafe {
        hostio::write_result(data.as_ptr(), data.len());
    }
}

/// Copies the bytes of the last EVM call or deployment return result.
/// Note: this function does not revert if out of bounds, but rather will copy the overlapping portion.
pub fn read_return_data(offset: usize, size: Option<usize>) -> Vec<u8> {
    let size = size.unwrap_or_else(|| return_data_len().saturating_sub(offset));

    let mut data = Vec::with_capacity(size);
    if size > 0 {
        unsafe {
            let bytes_written = hostio::read_return_data(data.as_mut_ptr(), offset, size);
            debug_assert!(bytes_written <= size);
            data.set_len(bytes_written);
        }
    };
    data
}

wrap_hostio!(
    /// Returns the length of the last EVM call or deployment return result, or `0` if neither have
    /// happened during the program's execution.
    return_data_len RETURN_DATA_LEN return_data_size usize
);

wrap_hostio!(
    /// Gets the address of the current program.
    address ADDRESS contract_address Address
);

/// Gets the balance of the current program.
pub fn balance() -> U256 {
    address().balance()
}
