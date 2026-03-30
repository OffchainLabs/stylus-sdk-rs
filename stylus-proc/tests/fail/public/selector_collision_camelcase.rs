// Copyright 2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Compilation should fail when two Rust functions produce the same Solidity
//! camelCase name (and thus the same selector) without any #[selector] override.
//! For example, `foo_bar` and `fooBar` both become `fooBar` in Solidity.

extern crate alloc;

use stylus_proc::public;

struct Contract {}

#[public]
#[allow(non_snake_case)]
impl Contract {
    fn foo_bar(_x: u64) {}

    fn fooBar(_x: u64) {}
}

fn main() {}
