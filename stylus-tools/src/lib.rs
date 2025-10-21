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

pub mod cargo_stylus;
#[cfg(feature = "integration-tests")]
pub mod devnet;
pub use cargo_stylus::*;
pub mod verifier;
pub use verifier::*;
pub mod exporter;
pub use error::{Error, Result};
pub use exporter::*;
