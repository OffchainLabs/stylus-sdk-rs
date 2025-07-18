// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::primitives::{Address, U256};
use stylus_tools::ops;

use crate::{
    common_args::{AuthArgs, ProviderArgs},
    error::CargoStylusResult,
};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Deployed and activated contract address to cache.
    address: Address,
    /// Bid, in wei, to place on the desired contract to cache. A value of 0 is a valid bid.
    bid: u64,

    #[command(flatten)]
    auth: AuthArgs,
    #[command(flatten)]
    provider: ProviderArgs,
}

pub async fn exec(args: Args) -> CargoStylusResult {
    let provider = args.provider.build_provider_with_wallet(&args.auth).await?;
    ops::cache::place_bid(
        args.address,
        U256::from(args.bid),
        args.auth.get_max_fee_per_gas_wei()?,
        &provider,
    )
    .await?;
    Ok(())
}
