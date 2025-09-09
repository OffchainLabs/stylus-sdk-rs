// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::path::PathBuf;

use alloy::{
    primitives::{Address, B256, U256},
    providers::{Provider, WalletProvider},
};
use cargo_metadata::{semver::Version, Package, TargetKind};

use crate::{
    core::{
        build::{build_contract, BuildConfig, BuildError},
        check::{check_contract, CheckConfig, CheckError},
        deployment::{deploy, DeploymentConfig, DeploymentError},
        manifest,
        reflection::ReflectionConfig,
    },
    error::decode_contract_error,
    ops,
    precompiles::{self, ArbWasm::ArbWasmErrors},
    utils::toolchain::get_toolchain_channel,
};

#[derive(Debug)]
pub struct Contract {
    // Metadata package
    pub package: Package,

    // Toolchain metadata
    stable: bool,

    // Cargo metadata
    name: String,
    version: Version,
}

impl Contract {
    pub fn is_contract(package: &Package) -> bool {
        if let Some(stylus_manifest_path) = package
            .manifest_path
            .parent()
            .map(|p| p.join(manifest::FILENAME))
        {
            stylus_manifest_path.exists()
        } else {
            false
        }
    }

    pub fn stable(&self) -> bool {
        self.stable
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    /// Checks whether a contract has already been activated with the most recent version of Stylus.
    pub async fn exists(codehash: B256, provider: &impl Provider) -> Result<bool, ContractError> {
        let arbwasm = precompiles::arb_wasm(provider);
        match arbwasm.codehashVersion(codehash).call().await {
            Ok(_) => Ok(true),
            Err(e) => {
                let errs = decode_contract_error(e)?;
                use ArbWasmErrors as A;
                match errs {
                    A::ProgramNotActivated(_)
                    | A::ProgramNeedsUpgrade(_)
                    | A::ProgramExpired(_) => Ok(false),
                    _ => Err(ContractError::UnexpectedArbWasmError),
                }
            }
        }
    }

    pub fn build(&self, config: &BuildConfig) -> Result<PathBuf, BuildError> {
        build_contract(self, config)
    }

    pub async fn check(
        &self,
        address: Option<Address>,
        config: &CheckConfig,
        provider: &impl Provider,
    ) -> Result<ContractStatus, CheckError> {
        check_contract(self, address, config, provider).await
    }

    pub async fn deploy(
        &self,
        config: &DeploymentConfig,
        provider: &(impl Provider + WalletProvider),
    ) -> Result<(), DeploymentError> {
        deploy(self, config, provider).await
    }

    pub fn export_abi(&self, config: &ReflectionConfig) -> eyre::Result<()> {
        ops::export_abi(self.package.name.as_ref(), config)
    }

    pub fn print_constructor(&self, config: &ReflectionConfig) -> eyre::Result<()> {
        ops::print_constructor(self.package.name.as_ref(), config)
    }
}

impl TryFrom<&Package> for Contract {
    type Error = ContractError;

    fn try_from(package: &Package) -> Result<Self, Self::Error> {
        let toolchain_channel = get_toolchain_channel(package)?;
        let stable = !toolchain_channel.contains("nightly");
        let version = package.version.clone();
        // First, let's try to find if the library's name is set, since this will interfere with
        // finding the wasm file in the deps directory if it's different.
        let name = package
            .targets
            .iter()
            .find_map(|t| t.kind.contains(&TargetKind::Lib).then(|| t.name.clone()))
            // If that doesn't work, then we can use the package name, and break normally.
            .unwrap_or_else(|| package.name.to_string());
        Ok(Self {
            package: package.clone(),
            stable,
            version,
            name,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ContractError {
    #[error("{0}")]
    ContractDecode(#[from] crate::error::ContractDecodeError),
    #[error("{0}")]
    Toolchain(#[from] crate::utils::toolchain::ToolchainError),

    #[error("unexpected ArbWasm error")]
    UnexpectedArbWasmError,
}

#[derive(Debug)]
pub enum ContractStatus {
    /// Contract already exists onchain.
    Active { code: Vec<u8> },
    /// Contract can be activated with the given data fee.
    Ready { code: Vec<u8>, fee: U256 },
}

impl ContractStatus {
    pub fn suggest_fee(&self) -> U256 {
        match self {
            Self::Active { .. } => U256::ZERO,
            Self::Ready { fee, .. } => *fee,
        }
    }
}
