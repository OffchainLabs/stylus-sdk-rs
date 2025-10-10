// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use crate::error::CargoStylusError;
use crate::{
    common_args::{
        ActivationArgs, AuthArgs, BuildArgs, CheckArgs, DeployArgs, ProjectArgs, ProviderArgs,
    },
    error::CargoStylusResult,
};
use alloy::primitives::{utils::parse_ether, Address, B256, U256};
use eyre::eyre;
use stylus_tools::core::deployment::deployer::ADDRESS;

// TODO: this should be in stylus-tools
pub const STYLUS_DEPLOYER_ADDRESS: Address = ADDRESS;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Only perform gas estimation.
    #[arg(long)]
    estimate_gas: bool,
    /// If specified, will not run the command in a reproducible docker container. Useful for local
    /// builds, but at the risk of not having a reproducible contract for verification purposes.
    #[arg(long)]
    no_verify: bool,
    /// Cargo stylus version when deploying reproducibly to downloads the corresponding cargo-stylus-base Docker image.
    /// If not set, uses the default version of the local cargo stylus binary.
    #[arg(long)]
    cargo_stylus_version: Option<String>,
    /// If set, do not activate the program after deploying it
    #[arg(long)]
    no_activate: bool,
    /// The address of the deployer contract that deploys, activates, and initializes the stylus constructor.
    #[arg(long, value_name = "DEPLOYER_ADDRESS", default_value_t = STYLUS_DEPLOYER_ADDRESS)]
    deployer_address: Address,
    /// The salt passed to the stylus deployer.
    #[arg(long, default_value_t = B256::ZERO)]
    deployer_salt: B256,
    /// The constructor arguments.
    #[arg(
        long,
        num_args(0..),
        value_name = "ARGS",
        allow_hyphen_values = true,
    )]
    constructor_args: Vec<String>,
    /// The amount of Ether sent to the contract through the constructor.
    #[arg(long, value_parser = parse_ether, default_value = "0")]
    constructor_value: U256,
    /// The constructor signature when using the --wasm-file flag.
    #[arg(long)]
    constructor_signature: Option<String>,

    /// Wallet source to use.
    #[command(flatten)]
    auth: AuthArgs,
    #[command(flatten)]
    activation: ActivationArgs,
    #[command(flatten)]
    build: BuildArgs,
    #[command(flatten)]
    check: CheckArgs,
    #[command(flatten)]
    deploy: DeployArgs,
    #[command(flatten)]
    project: ProjectArgs,
    #[command(flatten)]
    provider: ProviderArgs,
}

pub async fn exec(args: Args) -> CargoStylusResult {
    if args.project.contracts()?.len() > 1 && !args.constructor_args.is_empty() {
        return Err(CargoStylusError::from(eyre!(
            "Multi-contract deployment only allowed for no-arg constructors"
        )));
    }
    let provider = args.provider.build_provider_with_wallet(&args.auth).await?;
    let config = args.deploy.config(
        &args.activation,
        &args.check,
        args.auth.get_max_fee_per_gas_wei()?,
        args.estimate_gas,
        args.no_activate,
        args.deployer_address,
        args.constructor_args,
        args.deployer_salt,
        args.constructor_value,
    );
    for contract in args.project.contracts()? {
        contract.deploy(&config, &provider).await?;
    }
    Ok(())
}
