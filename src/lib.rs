// Copyright 2022-2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

pub use alloy_primitives;

use alloy_primitives::B256;

pub mod block;
pub mod contract;
pub mod crypto;
pub mod debug;
pub mod evm;
pub mod msg;
pub mod tx;
pub mod types;

mod hostio;

pub fn memory_grow(pages: u16) {
    unsafe { hostio::memory_grow(pages) }
}

pub fn args(len: usize) -> Vec<u8> {
    let mut input = Vec::with_capacity(len);
    unsafe {
        hostio::read_args(input.as_mut_ptr());
        input.set_len(len);
    }
    input
}

pub fn output(data: Vec<u8>) {
    unsafe {
        hostio::write_result(data.as_ptr(), data.len());
    }
}

#[macro_export]
macro_rules! entrypoint {
    ($name:expr) => {
        /// Force the compiler to import these symbols
        /// Note: calling these functions will unproductively consume gas
        #[no_mangle]
        pub unsafe fn mark_used() {
            stylus_sdk::memory_grow(0);
            panic!();
        }

        #[no_mangle]
        pub extern "C" fn arbitrum_main(len: usize) -> usize {
            let input = stylus_sdk::args(len);
            let (data, status) = match $name(input) {
                Ok(data) => (data, 0),
                Err(data) => (data, 1),
            };
            stylus_sdk::output(data);
            status
        }
    };
}

pub fn load_bytes32(key: B256) -> B256 {
    let mut data = B256::ZERO;
    unsafe { hostio::storage_load_bytes32(key.as_ptr(), data.as_mut_ptr()) };
    data
}

pub fn store_bytes32(key: B256, data: B256) {
    unsafe { hostio::storage_store_bytes32(key.as_ptr(), data.as_ptr()) };
}
