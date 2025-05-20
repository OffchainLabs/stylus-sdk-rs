// Copyright 2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

extern crate alloc;

use stylus_proc::public;

struct Contract {}

#[public]
impl Contract {
    // error: function can be only one of fallback, receive or constructor
    #[fallback]
    #[receive]
    #[constructor]
    fn init() {}
    
    // error: fallback, receive, and constructor can't have custom selector
    #[constructor]
    #[selector(name = "foo")]
    fn constr() {}

    // error: constructor function can only be defined using the corresponding attribute
    fn constructor() {}

    // error: constructor function can only be defined using the corresponding attribute
    #[receive]
    fn constructor() {}

    // error: constructor function can only be defined using the corresponding attribute
    fn stylus_constructor() {}

    // error: constructor function can only be defined using the corresponding attribute
    #[selector(name = "stylusConstructor")]
    fn foo() {}
}

fn main() {}

