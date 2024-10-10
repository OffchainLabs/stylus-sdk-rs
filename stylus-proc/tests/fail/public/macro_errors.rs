// Copyright 2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Compilation should fail for any unsupported attributes or other features.

extern crate alloc;

use stylus_proc::public;

struct Contract {}

#[public(unsupported)]
impl Contract {
    #[payable(unsupported)]
    fn test_method() {}
}

fn main() {}
