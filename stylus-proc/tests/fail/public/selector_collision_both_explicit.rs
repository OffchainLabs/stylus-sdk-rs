// Copyright 2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Compilation should fail when both functions carry explicit #[selector]
//! attributes that produce the same 4-byte ABI selector.

extern crate alloc;

use stylus_proc::public;

struct Contract {}

#[public]
impl Contract {
    #[selector(name = "doStuff")]
    fn alpha(_x: u64) {}

    #[selector(name = "doStuff")]
    fn beta(_x: u64) {}
}

fn main() {}
