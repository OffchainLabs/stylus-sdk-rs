// Copyright 2024-2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md
#![no_std]

//! Defines host environment methods Stylus SDK contracts have access to.

extern crate alloc;

pub mod calls;
pub mod deploy;
pub mod host;
pub mod storage;

pub use calls::*;
pub use host::*;
pub use storage::TopLevelStorage;
