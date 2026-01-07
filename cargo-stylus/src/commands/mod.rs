// Copyright 2025-2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use crate::error::CargoStylusResult;

mod activate;
mod build;
mod cache;
mod cgen;
mod check;
mod codehash_keepalive;
mod constructor;
mod debug_hook;
mod deploy;
mod export_abi;
mod get_initcode;
mod init;
mod new;
mod replay;
mod simulate;
mod trace;
mod usertrace;
mod verify;

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Activate an already deployed contract
    #[clap(visible_alias = "a")]
    Activate(activate::Args),
    /// Compile Stylus contracts
    #[clap(visible_alias = "b")]
    Build(build::Args),
    /// Cache a contract using the Stylus CacheManager for Arbitrum chains
    #[command(subcommand)]
    Cache(cache::Command),
    /// Generate c code bindings for a Stylus contract
    Cgen(cgen::Args),
    /// Check a contract
    #[clap(visible_alias = "c")]
    Check(check::Args),
    /// Request to keep contract from expiring in the codehash registry
    #[clap(visible_alias = "k")]
    CodehashKeepalive(codehash_keepalive::Args),
    /// Print the signature of a contract's constructor
    Constructor(constructor::Args),
    /// Deploy one or more Stylus contracts
    #[clap(visible_alias = "d")]
    Deploy(deploy::Args),
    /// Export a Solidity ABI
    ExportAbi(export_abi::Args),
    /// Generate and print initcode for a contract
    #[clap(visible_alias = "e")]
    GetInitcode(get_initcode::Args),
    /// Create a new Stylus project in an existing directory
    Init(init::Args),
    /// Create a new Stylus project
    New(new::Args),
    /// Replay a transaction using an external debugger
    #[clap(visible_alias = "r")]
    Replay(replay::Args),
    /// Simulate a transaction
    #[clap(visible_alias = "s")]
    Simulate(simulate::Args),
    /// Trace a transaction
    #[clap(visible_alias = "t")]
    Trace(trace::Args),
    /// Trace a transaction with stylusdb, capturing user function calls
    #[clap(visible_alias = "ut")]
    Usertrace(usertrace::Args),
    /// Verify the deployment of a Stylus contract
    Verify(verify::Args),
}

pub async fn exec(cmd: Command) -> CargoStylusResult {
    match cmd {
        Command::Activate(args) => activate::exec(args).await,
        Command::Build(args) => build::exec(args),
        Command::Cache(command) => cache::exec(command).await,
        Command::Cgen(args) => cgen::exec(args),
        Command::Check(args) => check::exec(args).await,
        Command::CodehashKeepalive(args) => codehash_keepalive::exec(args).await,
        Command::Constructor(args) => constructor::exec(args),
        Command::Deploy(args) => deploy::exec(args).await,
        Command::ExportAbi(args) => export_abi::exec(args),
        Command::GetInitcode(args) => get_initcode::exec(args),
        Command::Init(args) => init::exec(args),
        Command::New(args) => new::exec(args),
        Command::Replay(args) => replay::exec(args).await,
        Command::Simulate(args) => simulate::exec(args).await,
        Command::Trace(args) => trace::exec(args).await,
        Command::Usertrace(args) => usertrace::exec(args).await,
        Command::Verify(args) => verify::exec(args).await,
    }
}
