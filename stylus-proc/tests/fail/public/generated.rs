// Copyright 2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Compilation failures after macro generation completes.

extern crate alloc;

use stylus_proc::public;

struct UnsupportedType;

struct Contract {}

#[public]
impl Contract {
    fn unsupported_input(_arg: UnsupportedType) {}

    fn unsupported_output() -> UnsupportedType {
        UnsupportedType
    }
}

fn main() {}
