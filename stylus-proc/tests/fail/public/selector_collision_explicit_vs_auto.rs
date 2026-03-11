// Copyright 2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Compilation should fail when an explicit #[selector] on one function
//! produces the same 4-byte ABI selector as another function's auto-generated
//! selector.

extern crate alloc;

use stylus_proc::public;

struct Contract {}

#[public]
impl Contract {
    // Auto-generated ABI name: "fooBar(uint64)"
    fn foo_bar(_x: u64) {}

    // Explicit ABI name matching the auto-generated one: "fooBar(uint64)"
    #[selector(name = "fooBar")]
    fn something_else(_x: u64) {}
}

fn main() {}
