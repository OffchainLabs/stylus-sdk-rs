// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::primitives::TxHash;
use stylus_tools::core::tracing::{Trace, TraceConfig};

use crate::{common_args::ProviderArgs, error::CargoStylusResult};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Tx to replay.
    #[arg(short, long)]
    tx: TxHash,

    #[command(flatten)]
    provider: ProviderArgs,
    #[command(flatten)]
    config: TraceConfig,
}

pub async fn exec(args: Args) -> CargoStylusResult {
    let provider = args.provider.build_provider().await?;
    let trace = Trace::new(args.tx, &args.config, &provider)
        .await
        .map_err(eyre::Error::from)?;
    println!("{}", trace.json());
    Ok(())
}
