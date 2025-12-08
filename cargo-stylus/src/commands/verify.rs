// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::primitives::TxHash;
use eyre::eyre;
use itertools::izip;
use stylus_tools::core::build::reproducible::run_reproducible;

use crate::{
    common_args::{ProjectArgs, ProviderArgs, VerificationArgs},
    error::CargoStylusResult,
    utils::decode0x,
};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Cargo stylus version when deploying reproducibly to downloads the corresponding cargo-stylus-base Docker image.
    /// If not set, uses the default version of the local cargo stylus binary.
    #[arg(long)]
    cargo_stylus_version: Option<String>,
    /// Hash of the deployment transaction.
    #[arg(long)]
    deployment_tx: Vec<String>,

    #[arg(long)]
    skip_clean: bool,

    #[command(flatten)]
    project: ProjectArgs,
    #[command(flatten)]
    provider: ProviderArgs,
    #[command(flatten)]
    verification: VerificationArgs,
}

pub async fn exec(args: Args) -> CargoStylusResult {
    let provider = args.provider.build_provider().await?;

    for (contract, deployment_tx) in izip!(args.project.contracts()?, args.deployment_tx) {
        if args.verification.no_verify {
            let hash = decode0x(&deployment_tx)?;
            if hash.len() != 32 {
                return Err(eyre!("Invalid hash").into());
            }
            let hash = TxHash::from_slice(&hash);
            match contract.verify(hash, args.skip_clean, &provider).await? {
                stylus_tools::core::verification::VerificationStatus::Success => {
                    println!("Verification successful");
                }
                stylus_tools::core::verification::VerificationStatus::Failure(failure) => {
                    println!("Verification failed");
                    println!("prelude mismatch: {:?}", failure.prelude_mismatch);
                    println!("tx wasm length: {}", failure.tx_wasm_length);
                    println!("build wasm length: {}", failure.build_wasm_length);
                }
            }
        } else {
            println!("Running in a Docker container for reproducibility, this may take a while",);
            let mut cli_args: Vec<String> = vec![
                String::from("verify"),
                String::from("--no-verify"),
                String::from("--deployment-tx"),
            ];
            cli_args.push(deployment_tx);
            run_reproducible(
                &contract.package,
                args.cargo_stylus_version.clone(),
                cli_args,
            )?;
        }
    }

    Ok(())
}
