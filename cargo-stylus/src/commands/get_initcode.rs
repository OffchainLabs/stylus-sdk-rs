// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::{fs, path::PathBuf};

use eyre::Context;
use stylus_tools::ops;

use crate::{common_args::BuildArgs, error::CargoStylusResult};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// The output file - text file to store generated hex code.
    /// (defaults to stdout)
    #[arg(long)]
    output: Option<PathBuf>,

    #[command(flatten)]
    build: BuildArgs,
}

pub fn exec(args: Args) -> CargoStylusResult {
    let build_config = args.build.config();
    let writer = match args.output {
        Some(path) => fs::File::create(path).wrap_err("failed to create output file")?,
        None => todo!(),
    };
    ops::write_initcode(&build_config, writer)?;
    Ok(())
}
