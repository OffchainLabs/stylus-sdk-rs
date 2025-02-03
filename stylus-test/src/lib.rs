// Copyright 2025-2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! The Stylus testing suite.
//!
//! The stylus-test crate makes it easy to unit test all the storage types and contracts that use the
//! Stylus SDK. Included is an implementation of the [`stylus_core::host::Host`] trait that all Stylus
//! contracts have access to for interfacing with their host environment.
//!
//! The mock implementation, named [`crate::TestVM`], can be used to unit test Stylus contracts
//! in native Rust without the need for a real EVM or Arbitrum chain environment. [`crate::TestVM`]
//! allows for mocking of all host functions, including storage, gas, and external calls to assert
//! contract behavior.
//!
//! To be able to unit test Stylus contracts, contracts must access host methods through the [`stylus_core::host:HostAccessor`] trait,
//! which gives all contracts access to a `.vm()` method. That is, instead of calling `stylus_sdk::msg::value()` directly, contracts
//! should do `self.vm().msg_value()`. Global host function invocations are deprecated as of Stylus SDK 0.8.0, and contracts
//! should upgrade to the new host accessor pattern in preparation for 1.0.0.

pub mod builder;
pub mod constants;
pub mod state;
pub mod vm;
pub use builder::*;
pub use vm::*;
