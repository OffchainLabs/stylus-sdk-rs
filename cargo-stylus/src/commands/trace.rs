// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use stylus_tools::core::tracing::Trace;

use crate::{
    common_args::{ProviderArgs, TraceArgs},
    error::CargoStylusResult,
};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(flatten)]
    trace: TraceArgs,
    #[command(flatten)]
    provider: ProviderArgs,
}

pub async fn exec(args: Args) -> CargoStylusResult {
    let provider = args.provider.build_provider().await?;
    let trace = Trace::new(args.trace.tx, args.trace.use_native_tracer, &provider)
        .await
        .map_err(eyre::Error::from)?;
    println!("{}", trace.json());
    Ok(())
}
