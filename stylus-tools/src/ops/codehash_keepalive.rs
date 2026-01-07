// Copyright 2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::{
    primitives::B256,
    providers::{Provider, WalletProvider},
};
use eyre::eyre;

use crate::precompiles;

pub async fn codehash_keepalive(
    codehash: B256,
    provider: &(impl Provider + WalletProvider),
) -> eyre::Result<()> {
    precompiles::arb_wasm(provider)
        .codehashKeepalive(codehash)
        .call()
        .await
        .map_err(|err| eyre!("Failed to keepalive contract: {err:?}"))?;
    greyln!("Successfully kept alive contract with codehash {codehash}");
    Ok(())
}
