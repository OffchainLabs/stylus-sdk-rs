// Copyright 2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use stylus_proc::{storage, Erase};

#[storage]
#[derive(Erase)]
pub struct Erasable {}
