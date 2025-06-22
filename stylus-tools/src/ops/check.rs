// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::providers::Provider;

/// Checks that a contract is valid and can be deployed onchain.
///
/// Returns whether the WASM is already up-to-date and activated onchain, and the data fee.
pub fn check(provider: impl Provider) -> eyre::Result<()> {
    Ok(())
}
