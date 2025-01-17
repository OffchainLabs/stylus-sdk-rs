// Copyright 2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md
extern crate alloc;

use stylus_proc::{storage, Erase};
use stylus_sdk::storage::{StorageU256, StorageVec};

#[storage]
#[derive(Erase)]
pub struct Erasable {
    arr: StorageVec<StorageU256>,
}
