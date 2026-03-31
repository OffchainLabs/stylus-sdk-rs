// Copyright 2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::{
    primitives::B256,
    providers::{Provider, WalletProvider},
};
use eyre::{ensure, eyre};

use crate::{precompiles, utils::color::DebugColor};

pub async fn codehash_keepalive(
    codehash: B256,
    provider: &(impl Provider + WalletProvider),
) -> eyre::Result<()> {
    let arb_wasm = precompiles::arb_wasm(provider);
    let keepalive_call = arb_wasm.codehashKeepalive(codehash);

    keepalive_call
        .call()
        .await
        .map_err(|err| eyre!("Failed to keepalive contract: {err:?}"))?;

    let pending_tx = keepalive_call
        .send()
        .await
        .map_err(|err| eyre!("Failed to send keepalive transaction: {err:?}"))?;
    let receipt = pending_tx
        .get_receipt()
        .await
        .map_err(|err| eyre!("Failed to get keepalive transaction receipt: {err:?}"))?;
    ensure!(receipt.status(), "Keepalive transaction reverted");

    let tx_hash = receipt.transaction_hash.debug_lavender();
    greyln!("Successfully kept alive contract with codehash {codehash}");
    info!(@grey, "tx hash: {tx_hash}");
    Ok(())
}
