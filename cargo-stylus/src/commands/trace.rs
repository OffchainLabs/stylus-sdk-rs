// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::primitives::TxHash;
use stylus_tools::core::tracing::Trace;

use crate::{common_args::ProviderArgs, error::CargoStylusResult};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Tx to replay.
    #[arg(short, long)]
    tx: TxHash,
    /// If set, use the native tracer instead of the JavaScript one. Notice the native tracer might not be available in the node.
    #[arg(short, long, default_value_t = false)]
    use_native_tracer: bool,

    #[command(flatten)]
    provider: ProviderArgs,
}

pub async fn exec(args: Args) -> CargoStylusResult {
    let provider = args.provider.build_provider().await?;
    let trace = Trace::new(args.tx, args.use_native_tracer, &provider)
        .await
        .map_err(eyre::Error::from)?;
    println!("{}", trace.json());
    Ok(())
}
