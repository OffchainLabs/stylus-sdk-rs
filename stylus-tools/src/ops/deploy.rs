// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Contract deployment.

use alloy::providers::Provider;

/// Deploys a Stylus contract, activating if needed.
pub async fn deploy(_data_fee_bump_percent: u64, _provider: &impl Provider) -> eyre::Result<()> {
    //ops::check(data_fee_bump_percent, provider).await?;
    Ok(())
}
