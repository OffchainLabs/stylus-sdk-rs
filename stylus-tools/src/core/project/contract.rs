// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::path::PathBuf;

use alloy::{
    primitives::{Address, TxHash, B256, U256},
    providers::{Provider, WalletProvider},
};
use cargo_metadata::{semver::Version, Package, TargetKind};

use crate::{
    core::{
        build::{build_contract, BuildConfig, BuildError},
        check::{check_contract, CheckConfig, CheckError},
        code::{wasm::ProcessedWasm, Code},
        deployment::{deploy, DeploymentConfig, DeploymentError},
        manifest,
        reflection::ReflectionConfig,
        verification::{self, VerificationStatus},
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

    pub async fn verify(
        &self,
        tx_hash: TxHash,
        skip_clean: bool,
        provider: &impl Provider,
    ) -> eyre::Result<VerificationStatus> {
        let status = verification::verify(self, tx_hash, skip_clean, provider).await?;
        Ok(status)
    }

    pub fn print_constructor(&self, config: &ReflectionConfig) -> eyre::Result<()> {
        ops::print_constructor(self.package.name.as_ref(), config)
    }
}

/// Returns the name of the first library target, if any.
///
/// Cargo normalizes library names (e.g. hyphens become underscores) and names the
/// compiled artifact after them. Stylus contracts build as `cdylib`, which `cargo
/// metadata` reports as [`TargetKind::CDyLib`], so both kinds are accepted.
fn lib_target_name(targets: &[cargo_metadata::Target]) -> Option<String> {
    targets
        .iter()
        .find(|t| {
            t.kind
                .iter()
                .any(|k| matches!(k, TargetKind::Lib | TargetKind::CDyLib))
        })
        .map(|t| t.name.clone())
}

impl TryFrom<&Package> for Contract {
    type Error = ContractError;

    fn try_from(package: &Package) -> Result<Self, Self::Error> {
        let toolchain_channel = get_toolchain_channel(package)?;
        let stable = !toolchain_channel.contains("nightly");
        let version = package.version.clone();
        // Prefer the library target name: Cargo normalizes it (hyphens become
        // underscores) and that is the name of the wasm file in the deps directory.
        let name = lib_target_name(&package.targets).unwrap_or_else(|| package.name.to_string());
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
    Active { wasm: ProcessedWasm, code: Code },
    /// Contract can be activated with the given data fee.
    Ready {
        wasm: ProcessedWasm,
        code: Code,
        fee: U256,
    },
}

impl ContractStatus {
    pub fn wasm(&self) -> &ProcessedWasm {
        match &self {
            Self::Active { wasm, .. } => wasm,
            Self::Ready { wasm, .. } => wasm,
        }
    }

    pub fn code(&self) -> &Code {
        match &self {
            Self::Active { code, .. } => code,
            Self::Ready { code, .. } => code,
        }
    }

    pub fn suggest_fee(&self) -> U256 {
        match self {
            Self::Active { .. } => U256::ZERO,
            Self::Ready { fee, .. } => *fee,
        }
    }
}

#[cfg(test)]
mod tests {
    use cargo_metadata::Target;

    use super::*;

    fn target(kind: &[&str], name: &str) -> Target {
        serde_json::from_value(serde_json::json!({
            "kind": kind,
            "crate_types": kind,
            "name": name,
            "src_path": "/tmp/stylus-tools-test/src/lib.rs",
            "edition": "2021",
            "doc": true,
            "doctest": true,
            "test": true,
        }))
        .unwrap()
    }

    #[test]
    fn lib_target_name_for_cdylib() {
        // `crate-type = ["cdylib", "rlib"]` is reported as CDyLib/RLib by cargo
        // metadata, never as Lib; the package name may contain hyphens while the
        // artifact name always uses underscores.
        let targets = [target(&["cdylib", "rlib"], "nimbus_contracts")];
        assert_eq!(
            lib_target_name(&targets).as_deref(),
            Some("nimbus_contracts")
        );
    }

    #[test]
    fn lib_target_name_for_plain_lib() {
        let targets = [target(&["lib"], "my_contract")];
        assert_eq!(lib_target_name(&targets).as_deref(), Some("my_contract"));
    }

    #[test]
    fn lib_target_name_without_library_target() {
        let targets = [target(&["bin"], "nimbus-contracts")];
        assert_eq!(lib_target_name(&targets), None);
    }
}
