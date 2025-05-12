// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Contract acitvation.
//!
//! See the [Arbitrum Docs](https://docs.arbitrum.io/stylus/concepts/how-it-works#activation) for
//! more info on contract activation.

use alloy::{primitives::Address, providers::Provider};

use crate::error::Result;

/// Activates an already deployed Stylus contract by address.
pub async fn activate_contract(address: Address, provider: &impl Provider) -> Result<()> {
    let _code = provider.get_code_at(address).await?;
    Ok(())
}
