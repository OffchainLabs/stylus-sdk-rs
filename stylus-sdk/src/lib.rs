// Copyright 2022-2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

pub use alloy_primitives;
pub use alloy_sol_types;
pub use hex;
pub use keccak_const;
pub use stylus_proc;

#[macro_use]
pub mod abi;

#[macro_use]
pub mod debug;

pub mod block;
pub mod call;
pub mod contract;
pub mod crypto;
pub mod deploy;
pub mod evm;
pub mod msg;
pub mod prelude;
pub mod storage;
pub mod tx;
pub mod types;
pub mod util;

mod hostio;

/// Represents a contract invocation outcome
pub type ArbResult = Result<Vec<u8>, Vec<u8>>;

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
        stylus_sdk::entrypoint!($name, false);
    };
    ($name:expr, $allow_reentrant:expr) => {
        /// Force the compiler to import these symbols
        /// Note: calling this function will unproductively consume gas
        #[no_mangle]
        pub unsafe fn mark_used() {
            stylus_sdk::memory_grow(0);
            panic!();
        }

        #[no_mangle]
        pub extern "C" fn user_entrypoint(len: usize) -> usize {
            if !$allow_reentrant && stylus_sdk::msg::reentrant() {
                return 1; // revert on reentrancy
            }
            if $allow_reentrant {
                unsafe { stylus_sdk::call::opt_into_reentrancy() };
            }

            let input = stylus_sdk::args(len);
            let (data, status) = match $name(input) {
                Ok(data) => (data, 0),
                Err(data) => (data, 1),
            };
            stylus_sdk::storage::StorageCache::flush();
            stylus_sdk::output(data);
            status
        }
    };
}
