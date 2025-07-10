// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;

use alloy_primitives::U256;
use stylus_sdk::{storage::StorageU256, host::VMAccess, prelude::*};

#[entrypoint]
#[storage]
pub struct Contract {
}

#[public]
impl Contract {
    pub fn set_number(&mut self, number: U256) {
        unsafe {
            get_storage_slot(self).set(number);
        }
    }

    pub fn number(&self) -> U256 {
        unsafe {
            get_storage_slot(self).get()
        }
    }
}

unsafe fn get_storage_slot<VMA: VMAccess>(vma: &VMA) -> StorageU256 {
    StorageU256::new(U256::ZERO, 0, vma.raw_vm())
}
