// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::primitives::Address;
use stylus_tools::ops;

use crate::{common_args::ProviderArgs, error::CargoStylusResult};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Stylus contract address to suggest a minimum bid for in the cache manager.
    address: Address,

    #[command(flatten)]
    provider: ProviderArgs,
}

pub async fn exec(args: Args) -> CargoStylusResult {
    let provider = args.provider.build_provider().await?;
    ops::cache::suggest_bid(args.address, &provider).await?;
    Ok(())
}
