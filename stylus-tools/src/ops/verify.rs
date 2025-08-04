// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::{primitives::TxHash, providers::Provider};

//use crate::core;

pub async fn verify(_tx_hash: TxHash, _provider: &impl Provider) -> eyre::Result<()> {
    //core::verification::verify(tx_hash, provider).await?;
    Ok(())
}
