// Copyright 2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use crate::{
    hostio::{self, wrap_hostio, RETURN_DATA_SIZE},
    types::AddressVM,
};
use alloy_primitives::{Address, B256};

pub fn read_return_data(offset: usize, size: Option<usize>) -> Vec<u8> {
    let size = unsafe { size.unwrap_or_else(|| RETURN_DATA_SIZE.get().saturating_sub(offset)) };

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
    return_data_len return_data_size usize
);

wrap_hostio!(
    /// Gets the address of the current program.
    address contract_address Address
);

/// Gets the balance of the current program.
pub fn balance() -> B256 {
    address().balance()
}
