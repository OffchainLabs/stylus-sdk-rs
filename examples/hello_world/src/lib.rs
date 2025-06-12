// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;

use stylus_sdk::{console, prelude::*, stylus_proc::entrypoint, ArbResult};

#[storage]
#[entrypoint]
pub struct Hello;

#[public]
impl Hello {
    fn user_main(_input: Vec<u8>) -> ArbResult {
        // Will print 'Stylus says: Hello Stylus!' on your local dev node
        // Be sure to add "debug" feature flag to your Cargo.toml file as
        // shown below.
        console!("Hello Stylus!");
        Ok(Vec::new())
    }
}
