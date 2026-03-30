// Copyright 2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::primitives::B256;
use stylus_tools::ops;

use crate::{
    common_args::{AuthArgs, ProviderArgs},
    error::CargoStylusResult,
};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Deployed Stylus contract codehash to keepalive
    #[arg(long)]
    codehash: B256,
    #[command(flatten)]
    auth: AuthArgs,
    #[command(flatten)]
    provider: ProviderArgs,
}

pub async fn exec(args: Args) -> CargoStylusResult {
    let provider = args.provider.build_provider_with_wallet(&args.auth).await?;
    ops::codehash_keepalive::codehash_keepalive(args.codehash, &provider).await?;
    Ok(())
}
