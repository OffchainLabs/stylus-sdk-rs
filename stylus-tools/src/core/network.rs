// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use crate::utils::color::Color;

#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("The old Stylus testnet is no longer supported.\nPlease downgrade to {}", "cargo stylus version 0.2.1".red())]
    TestnetNotSupported,
}

pub fn check_endpoint(endpoint: &str) -> Result<(), NetworkError> {
    if endpoint == "https://stylus-testnet.arbitrum.io/rpc" {
        Err(NetworkError::TestnetNotSupported)
    } else {
        Ok(())
    }
}
