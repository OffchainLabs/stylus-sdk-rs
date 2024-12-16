// Copyright 2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

extern crate alloc;

use stylus_proc::public;

struct Contract {}

#[public]
impl Contract {
    #[constructor]
    #[selector(name = "foo")]
    #[fallback]
    fn init() {}

    // constructor without annotation
    fn constructor() {}
}

fn main() {}

