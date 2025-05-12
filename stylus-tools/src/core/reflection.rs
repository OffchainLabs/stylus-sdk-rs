// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Get information about a Stylus contract at build time
//!
//! This uses the mechanism of running a Stylus contract crate as a binary to return information
//! about the contract. This does not depend on a deployment of the contract.

use alloy::json_abi::JsonAbi;

use crate::Result;

/// Export a Solidity ABI for a Stylus contract
pub fn abi() -> Result<JsonAbi> {
    todo!()
}
