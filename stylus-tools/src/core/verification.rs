// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::{primitives::TxHash, providers::Provider};

use crate::utils::cargo;

pub async fn verify(tx_hash: TxHash, provider: &impl Provider) -> Result<bool, VerificationError> {
    let _tx = provider
        .get_transaction_by_hash(tx_hash)
        .await?
        .ok_or(VerificationError::NoCodeAtAddress)?;
    cargo::clean()?;
    Ok(false)
}

#[derive(Debug, thiserror::Error)]
pub enum VerificationError {
    #[error("RPC failed: {0}")]
    Rpc(#[from] alloy::transports::RpcError<alloy::transports::TransportErrorKind>),

    #[error("{0}")]
    Command(#[from] crate::error::CommandError),

    #[error("No code at address")]
    NoCodeAtAddress,
}
