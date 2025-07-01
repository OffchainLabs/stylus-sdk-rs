// Copyright 2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

// contract-client-gen feature can generate code that makes some imports of this file unused
#![allow(unused_imports)]

extern crate alloc;

use stylus_proc::{storage, Erase};
use stylus_sdk::prelude::*;
use stylus_sdk::storage::{StorageType, StorageU256, StorageVec};

#[storage]
#[derive(Erase)]
pub struct Erasable {
    arr: StorageVec<StorageU256>,
}
