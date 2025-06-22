// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::primitives::Address;
use stylus_tools::ops;

use crate::{
    common_args::{AuthArgs, DataFeeArgs, ProviderArgs},
    error::CargoStylusResult,
};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Deployed Stylus contract address to activate
    #[arg(long)]
    address: Address,
    /// Whether or not to just estimate gas without sending a tx
    #[arg(long)]
    estimate_gas: bool,

    #[command(flatten)]
    auth: AuthArgs,
    #[command(flatten)]
    data_fee: DataFeeArgs,
    #[command(flatten)]
    provider: ProviderArgs,
}

pub async fn exec(args: Args) -> CargoStylusResult {
    let provider = args.provider.build_provider_with_wallet(&args.auth).await?;
    if args.estimate_gas {
        ops::activation_estimate_gas(args.address, args.data_fee.bump_percent, &provider).await?;
    } else {
        ops::activate_contract(args.address, args.data_fee.bump_percent, &provider).await?;
    }
    Ok(())
}
