// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::primitives::TxHash;
use eyre::eyre;
use stylus_tools::ops;

use crate::{
    common_args::{ProviderArgs, VerificationArgs},
    error::CargoStylusResult,
    utils::decode0x,
};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Hash of the deployment transaction.
    #[arg(long)]
    deployment_tx: String,

    #[command(flatten)]
    provider: ProviderArgs,
    #[command(flatten)]
    verification: VerificationArgs,
}

pub async fn exec(args: Args) -> CargoStylusResult {
    let provider = args.provider.build_provider().await?;
    let hash = decode0x(args.deployment_tx)?;
    if hash.len() != 32 {
        return Err(eyre!("Invalid hash").into());
    }
    let hash = TxHash::from_slice(&hash);
    ops::verify(hash, &provider).await?;
    Ok(())
}
