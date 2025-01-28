// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Deploy other contracts.
//!
//! Currently this module only supports low-level contract creation via [`RawDeploy`],
//! but work is being done to introduce high-level deployment patterns.

#[allow(deprecated)]
pub use raw::RawDeploy;

mod raw;
