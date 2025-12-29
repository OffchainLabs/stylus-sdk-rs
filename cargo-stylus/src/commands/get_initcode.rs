// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::{fs, io, path::PathBuf};

use eyre::Context;
use stylus_tools::ops;

use crate::{
    common_args::{BuildArgs, ProjectArgs},
    error::CargoStylusResult,
};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// The output file - text file to store generated hex code.
    /// (defaults to stdout)
    #[arg(long)]
    output: Option<PathBuf>,

    #[command(flatten)]
    build: BuildArgs,
    #[command(flatten)]
    project: ProjectArgs,
}

pub fn exec(args: Args) -> CargoStylusResult {
    let writer = match args.output {
        Some(path) => &mut fs::File::create(path).wrap_err("failed to create output file")?
            as &mut dyn io::Write,
        None => &mut io::stdout(),
    };
    let build_config = args.build.config();
    let project_config = args.project.config();
    for contract in args.project.contracts()? {
        ops::write_initcode(&contract, &build_config, &project_config, &mut *writer)?;
    }
    Ok(())
}
