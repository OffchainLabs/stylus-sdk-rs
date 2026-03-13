// Copyright 2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Compilation should fail when three or more functions in the same #[public]
//! block produce the same 4-byte ABI selector. All pairwise collisions should
//! be reported.

extern crate alloc;

use stylus_proc::public;

struct Contract {}

#[public]
impl Contract {
    fn foo(_x: u64) {}

    #[selector(name = "foo")]
    fn bar(_x: u64) {}

    #[selector(name = "foo")]
    fn baz(_x: u64) {}
}

fn main() {}
