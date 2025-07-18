// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Tools for working with stylus projects.
// TODO: #![doc = include_str!("../README.md")]

#[macro_use]
mod macros;

pub mod core;
pub(crate) mod error;
pub mod manifest;
pub mod ops;
pub mod precompiles;
pub mod verification;
pub mod wasm;

pub mod utils;

#[cfg(feature = "integration-tests")]
pub mod devnet;

pub mod cargo_stylus;

pub use cargo_stylus::*;
pub use error::{Error, Result};
