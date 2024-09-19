// Copyright 2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Compilation will fail if the type is not wrapped in the [`sol!`][alloy_sol_types::sol] macro.

use stylus_proc::AbiType;
use stylus_sdk::storage::StorageBool;

#[derive(AbiType)]
struct MyStruct {
    bar: StorageBool,
}

fn main() {}
