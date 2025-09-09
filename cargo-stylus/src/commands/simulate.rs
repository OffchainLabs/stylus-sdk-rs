// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use stylus_tools::core::tracing::{SimulateConfig, Trace};

use crate::{common_args::ProviderArgs, error::CargoStylusResult};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(flatten)]
    provider: ProviderArgs,
    #[command(flatten)]
    config: SimulateConfig,
}

pub async fn exec(args: Args) -> CargoStylusResult {
    let provider = args.provider.build_provider().await?;
    let trace = Trace::simulate(&args.config, &provider)
        .await
        .map_err(eyre::Error::from)?;
    println!("{}", trace.json());
    Ok(())
}
