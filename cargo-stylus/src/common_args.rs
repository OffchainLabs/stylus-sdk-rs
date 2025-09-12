// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::{fs, path::PathBuf};

use alloy::{
    network::EthereumWallet,
    primitives::FixedBytes,
    providers::{Provider, ProviderBuilder, WalletProvider},
    signers::{
        local::{LocalSigner, PrivateKeySigner},
        Signer,
    },
};
use cargo_util_schemas::manifest::PackageName;
use eyre::{eyre, Context};
use stylus_tools::core::{
    activation::ActivationConfig,
    build::BuildConfig,
    check::CheckConfig,
    deployment::DeploymentConfig,
    project::{contract::Contract, workspace::Workspace},
    reflection::ReflectionConfig,
};

use crate::{
    constants::DEFAULT_ENDPOINT,
    utils::{convert_gwei_to_wei, decode0x},
};

#[derive(Debug, clap::Args)]
pub struct ActivationArgs {
    /// Percent to bump the estimated activation data fee by
    #[arg(long, default_value = "20")]
    pub data_fee_bump_percent: u64,
}

impl ActivationArgs {
    pub fn config(&self) -> ActivationConfig {
        ActivationConfig {
            data_fee_bump_percent: self.data_fee_bump_percent,
        }
    }
}

#[derive(Debug, clap::Args)]
pub struct AuthArgs {
    /// File path to a text file containing a hex-encoded private key
    #[arg(long)]
    private_key_path: Option<PathBuf>,
    /// Private key as a hex string. Warning: this exposes your key to shell history
    #[arg(long)]
    private_key: Option<String>,
    /// Path to an Ethereum wallet keystore file (e.g. clef)
    #[arg(long)]
    keystore_path: Option<String>,
    /// Keystore password file
    #[arg(long)]
    keystore_password_path: Option<PathBuf>,
    /// Optional max fee per gas in gwei units.
    #[arg(long)]
    max_fee_per_gas_gwei: Option<String>,
}

impl AuthArgs {
    fn build_wallet(&self, chain_id: u64) -> eyre::Result<EthereumWallet> {
        if let Some(key) = &self.private_key {
            if key.is_empty() {
                return Err(eyre!("empty private key"));
            }
            let priv_key_bytes: FixedBytes<32> = FixedBytes::from_slice(decode0x(key)?.as_slice());
            let signer =
                PrivateKeySigner::from_bytes(&priv_key_bytes)?.with_chain_id(Some(chain_id));
            return Ok(EthereumWallet::new(signer));
        }

        if let Some(file) = &self.private_key_path {
            let key = fs::read_to_string(file).wrap_err("could not open private key file")?;
            let priv_key_bytes: FixedBytes<32> = FixedBytes::from_slice(decode0x(key)?.as_slice());
            let signer =
                PrivateKeySigner::from_bytes(&priv_key_bytes)?.with_chain_id(Some(chain_id));
            return Ok(EthereumWallet::new(signer));
        }

        let keystore = self.keystore_path.as_ref().ok_or(eyre!("no keystore"))?;
        let password = self
            .keystore_password_path
            .as_ref()
            .map(fs::read_to_string)
            .unwrap_or(Ok("".into()))?;

        let signer =
            LocalSigner::decrypt_keystore(keystore, password)?.with_chain_id(Some(chain_id));
        Ok(EthereumWallet::new(signer))
    }

    pub fn get_max_fee_per_gas_wei(&self) -> eyre::Result<Option<u128>> {
        self.max_fee_per_gas_gwei
            .as_ref()
            .map(|fee_str| convert_gwei_to_wei(fee_str))
            .transpose()
    }
}

#[derive(Debug, clap::Args)]
pub struct BuildArgs {
    /// Specifies the features to use when building the Stylus binary.
    #[arg(long)]
    pub features: Vec<String>,
    /// The path to source files to include in the project hash, which is included in the contract
    /// deployment init code transaction to be used for verification of deployment integrity.
    ///
    /// If not provided, all .rs files and Cargo.toml and Cargo.lock files in project's directory
    /// tree are included.
    // TODO: where is this default set?
    #[arg(long)]
    source_files_for_project_hash: Vec<String>,
}

impl BuildArgs {
    pub fn config(&self) -> BuildConfig {
        BuildConfig {
            features: self.features.clone(),
            ..Default::default()
        }
    }
}

#[derive(Debug, clap::Args)]
pub struct CheckArgs {}

impl CheckArgs {
    pub fn config(&self, activation: &ActivationArgs) -> CheckConfig {
        CheckConfig {
            activation: activation.config(),
            ..Default::default()
        }
    }
}

#[derive(Debug, clap::Args)]
pub struct DeployArgs {}

impl DeployArgs {
    pub fn config(&self, activate: &ActivationArgs, check: &CheckArgs) -> DeploymentConfig {
        DeploymentConfig {
            check: check.config(activate),
            ..Default::default()
        }
    }
}

#[derive(Debug, clap::Args)]
pub struct ProjectArgs {
    #[arg(long)]
    contract: Vec<PackageName>,
}

impl ProjectArgs {
    pub fn contracts(&self) -> eyre::Result<Vec<Contract>> {
        let workspace = Workspace::current()?;
        if self.contract.is_empty() {
            Ok(workspace
                .default_contracts()
                .collect::<Result<Vec<_>, _>>()?)
        } else {
            workspace
                .filter_contracts(self.contract.clone().into_iter())
                .map_err(Into::into)
        }
    }
}

#[derive(Debug, clap::Args)]
pub struct ProviderArgs {
    /// Arbitrum RPC endpoint
    #[arg(short, long, default_value = DEFAULT_ENDPOINT)]
    pub endpoint: String,
}

impl ProviderArgs {
    pub async fn build_provider(&self) -> eyre::Result<impl Provider> {
        let provider = ProviderBuilder::new().connect(&self.endpoint).await?;
        Ok(provider)
    }

    pub async fn build_provider_with_wallet(
        &self,
        auth: &AuthArgs,
    ) -> eyre::Result<impl Provider + WalletProvider> {
        let provider = self.build_provider().await?;
        let chain_id = provider.get_chain_id().await?;
        let wallet = auth.build_wallet(chain_id)?;
        let provider = ProviderBuilder::new()
            .wallet(wallet)
            .connect(&self.endpoint)
            .await?;
        Ok(provider)
    }
}

#[derive(Debug, clap::Args)]
pub struct ReflectionArgs {
    /// The output file (defaults to stdout).
    #[arg(long)]
    output: Option<PathBuf>,
    /// Write a JSON ABI instead using solc. Requires solc.
    #[arg(long)]
    json: bool,
    /// Rust crate's features list. Required to include feature specific abi.
    #[arg(long)]
    rust_features: Option<Vec<String>>,
}

impl ReflectionArgs {
    pub fn config(&self) -> ReflectionConfig {
        ReflectionConfig {
            file: self.output.clone(),
            json: self.json,
            rust_features: self.rust_features.clone(),
        }
    }
}

#[derive(Debug, clap::Args)]
pub struct VerificationArgs {
    /// If specified, will not run the command in a reproducible docker container.
    ///
    /// Useful for local builds, but at the risk of not having a reproducible contract for
    /// verification purposes.
    #[arg(long)]
    no_verify: bool,
}
