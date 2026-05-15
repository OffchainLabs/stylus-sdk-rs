// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::path::PathBuf;

use alloy::primitives::{utils::parse_ether, Address, B256, U256};
use eyre::eyre;
use stylus_tools::core::{
    build::reproducible::run_reproducible, deployment, deployment::deployer::ADDRESS,
};

use crate::{
    common_args::{
        ActivationArgs, AuthArgs, BuildArgs, CheckArgs, DeployArgs, ProjectArgs, ProviderArgs,
    },
    error::{CargoStylusError, CargoStylusResult},
};

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
    /// Cargo stylus version when deploying reproducibly to downloads the corresponding
    /// cargo-stylus-base Docker image. If not set, uses the default version of the local cargo
    /// stylus binary.
    #[arg(long)]
    cargo_stylus_version: Option<String>,
    /// If set, do not activate the program after deploying it
    #[arg(long)]
    no_activate: bool,
    /// The address of the deployer contract that deploys, activates, and initializes the stylus
    /// constructor.
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
    /// Deploy wasm file directly
    #[arg(long)]
    wasm_file: Option<PathBuf>,

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

impl Args {
    /// Reconstruct CLI flags for forwarding to a Docker invocation.
    /// Does not include `--no-verify`, `--contract`, or `--cargo-stylus-version`
    /// as those are handled by the caller.
    fn to_cli_args(&self) -> Vec<String> {
        let mut args = vec![String::from("-e"), self.provider.endpoint.clone()];
        args.extend(self.auth.to_cli_args());
        if self.estimate_gas {
            args.push("--estimate-gas".into());
        }
        if self.no_activate {
            args.push("--no-activate".into());
        }
        if self.deployer_address != STYLUS_DEPLOYER_ADDRESS {
            args.extend([
                "--deployer-address".into(),
                self.deployer_address.to_string(),
            ]);
        }
        if self.deployer_salt != B256::ZERO {
            args.extend(["--deployer-salt".into(), self.deployer_salt.to_string()]);
        }
        if !self.constructor_args.is_empty() {
            args.push("--constructor-args".into());
            args.extend(self.constructor_args.clone());
        }
        if self.constructor_value != U256::ZERO {
            args.extend([
                "--constructor-value".into(),
                self.constructor_value.to_string(),
            ]);
        }
        if let Some(sig) = &self.constructor_signature {
            args.extend(["--constructor-signature".into(), sig.clone()]);
        }
        for feature in &self.build.features {
            args.extend(["--features".into(), feature.clone()]);
        }
        args
    }
}

pub async fn exec(args: Args) -> CargoStylusResult {
    let contracts = args.project.contracts()?;

    if !args.no_verify {
        // Run inside a Docker container for reproducible builds.
        println!("Running in a Docker container for reproducibility, this may take a while");
        for contract in &contracts {
            let mut cli_args = vec![
                String::from("deploy"),
                String::from("--no-verify"),
                String::from("--contract"),
                contract.package.name.to_string(),
            ];
            cli_args.extend(args.to_cli_args());
            run_reproducible(
                &contract.package,
                args.cargo_stylus_version.clone(),
                cli_args,
            )?;
        }
        return Ok(());
    }

    // Direct deployment (no Docker).
    if args.wasm_file.is_none() && contracts.len() > 1 && !args.constructor_args.is_empty() {
        return Err(CargoStylusError::from(eyre!(
            "Multi-contract deployment only allowed for no-arg constructors"
        )));
    }
    let provider = args.provider.build_provider_with_wallet(&args.auth).await?;
    let config = args.deploy.config(
        &args.activation,
        &args.build,
        &args.check,
        args.auth.get_max_fee_per_gas_wei()?,
        args.no_activate,
        args.deployer_address,
        args.constructor_args,
        args.deployer_salt,
        args.constructor_value,
    );
    #[allow(clippy::collapsible_else_if)]
    if args.estimate_gas {
        if let Some(wasm_file) = args.wasm_file {
            let _gas = deployment::estimate_gas_wasm_file(wasm_file, &config, &provider).await?;
        } else {
            for contract in contracts {
                let _gas = deployment::estimate_gas(&contract, &config, &provider).await?;
            }
        }
    } else {
        if let Some(wasm_file) = args.wasm_file {
            deployment::deploy_wasm_file(wasm_file, &config, &provider).await?;
        } else {
            for contract in contracts {
                contract.deploy(&config, &provider).await?;
            }
        }
    }
    Ok(())
}
