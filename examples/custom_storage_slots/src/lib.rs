// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(feature = "contract-client-gen", allow(unused_imports))]

extern crate alloc;

use alloy_primitives::U256;
use stylus_sdk::{host::VMAccess, prelude::*, storage::StorageU256};

#[entrypoint]
#[storage]
pub struct Contract {}

#[public]
impl Contract {
    pub fn set_number(&mut self, number: U256) {
        unsafe {
            get_storage_slot(self).set(number);
        }
    }

    pub fn number(&self) -> U256 {
        unsafe { get_storage_slot(self).get() }
    }
}

#[cfg(not(feature = "contract-client-gen"))]
unsafe fn get_storage_slot<VMA: VMAccess>(vma: &VMA) -> StorageU256 {
    StorageU256::new(U256::ZERO, 0, vma.raw_vm())
}
