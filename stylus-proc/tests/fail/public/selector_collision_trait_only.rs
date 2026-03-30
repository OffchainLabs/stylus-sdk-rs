// Copyright 2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Compilation should fail when two functions in a `#[public]` trait definition
//! produce the same 4-byte ABI selector, even without a corresponding impl.

extern crate alloc;

use stylus_proc::public;

#[public]
trait TraitWithCollision {
    fn foo(&self, _x: u64);

    #[selector(name = "foo")]
    fn colliding_foo(&self, _x: u64);
}

fn main() {}
