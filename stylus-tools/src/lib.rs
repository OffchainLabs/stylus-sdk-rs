// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Tools for working with stylus projects.
// TODO: #![doc = include_str!("../README.md")]

#[macro_use]
mod macros;

pub mod activate;
pub mod check;
pub mod contract;
pub mod error;
pub mod manifest;
pub mod ops;
pub mod precompiles;
pub mod verify;
pub mod wasm;
pub mod workspace;

pub(crate) mod cargo;
pub(crate) mod utils;

#[cfg(feature = "integration-tests")]
pub mod devnet;

pub mod cargo_stylus;

pub use cargo_stylus::*;
