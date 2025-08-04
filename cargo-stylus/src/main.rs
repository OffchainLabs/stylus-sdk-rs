// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! CLI for `cargo-stylus`.

use std::process::ExitCode;

use clap::Parser;

mod commands;
mod common_args;
mod constants;
mod error;
mod utils;

#[derive(Debug, Parser)]
#[command(name = "stylus")]
#[command(bin_name = "cargo stylus")]
#[command(author = "Offchain Labs, Inc.")]
#[command(about = "Cargo subcommand for developing Stylus projects", long_about = None)]
#[command(propagate_version = true)]
#[command(version)]
struct Args {
    #[command(subcommand)]
    command: commands::Command,

    /// Whether to print debug info.
    #[arg(long, global = true)]
    verbose: bool,
}

fn main() -> ExitCode {
    // Parse args from CLI, skipping `stylus` arg coming from `cargo`
    let args: Vec<_> = std::env::args().skip(1).collect();
    let args = Args::parse_from(args);

    let log_level = if args.verbose {
        log::Level::Debug
    } else {
        log::Level::Info
    };
    simple_logger::init_with_level(log_level).expect("setting up logger");

    // Create the async runtime
    let mut runtime = match args.command {
        // Use the current thread for replay
        commands::Command::Replay(_) => tokio::runtime::Builder::new_current_thread(),
        _ => tokio::runtime::Builder::new_multi_thread(),
    };

    // Build async runtime and block on command execution
    let result = runtime
        .enable_all()
        .build()
        .map_err(Into::into)
        .and_then(|rt| rt.block_on(commands::exec(args.command)));

    // Report any error and return proper exit code
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            utils::print_error(&err);
            err.exit_code()
        }
    }
}
