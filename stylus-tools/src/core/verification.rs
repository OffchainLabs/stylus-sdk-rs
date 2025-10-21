// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use crate::core::deployment::deployer::stylus_constructorCall;
use crate::core::deployment::deployer::StylusDeployer::deployCall;
use crate::core::verification::VerificationError::InvalidInitData;
use crate::core::{
    deployment::prelude::DeploymentCalldata, project::contract::Contract, reflection,
};
use alloy::sol_types::SolCall;
use alloy::{
    consensus::Transaction,
    primitives::{Address, TxHash},
    providers::Provider,
};

pub async fn verify(
    contract: &Contract,
    tx_hash: TxHash,
    provider: &impl Provider,
) -> Result<VerificationStatus, VerificationError> {
    let tx = provider
        .get_transaction_by_hash(tx_hash)
        .await?
        .ok_or(VerificationError::NoCodeAtAddress)?;
    // cargo::clean()?;
    let status = contract.check(None, &Default::default(), provider).await?;

    let constructor_called = deployCall::abi_decode(tx.input())
        .unwrap()
        .initData
        .starts_with(stylus_constructorCall::SELECTOR.as_slice());
    if !constructor_called {
        return Err(InvalidInitData);
    }

    let deployment_data = DeploymentCalldata::new(status.code());
    let calldata = DeploymentCalldata(tx.input().to_vec());
    if let Some(deployer_address) = tx.to() {
        verify_constructor_deployment(&calldata, &deployment_data, deployer_address)
    } else {
        Ok(verify_create_deployment(&calldata, &deployment_data))
    }
}

fn verify_create_deployment(
    calldata: &DeploymentCalldata,
    deployment_data: &DeploymentCalldata,
) -> VerificationStatus {
    if deployment_data == calldata {
        return VerificationStatus::Success;
    }

    let tx_prelude = calldata.prelude();
    let build_prelude = calldata.prelude();
    let prelude_mismatch = if tx_prelude == build_prelude {
        None
    } else {
        Some(PreludeMismatch {
            tx: hex::encode(tx_prelude),
            build: hex::encode(build_prelude),
        })
    };

    let tx_wasm_length = deployment_data.compressed_wasm().len();
    let build_wasm_length = calldata.compressed_wasm().len();
    VerificationStatus::Failure(VerificationFailure {
        prelude_mismatch,
        tx_wasm_length,
        build_wasm_length,
    })
}

fn verify_constructor_deployment(
    _calldata: &DeploymentCalldata,
    _deployment_data: &DeploymentCalldata,
    _deployer_address: Address,
) -> Result<VerificationStatus, VerificationError> {
    let _constructor = reflection::constructor()?.ok_or(VerificationError::NoConstructor)?;
    todo!()
}

#[derive(Debug)]
pub enum VerificationStatus {
    Success,
    Failure(VerificationFailure),
}

#[derive(Debug)]
pub struct VerificationFailure {
    pub prelude_mismatch: Option<PreludeMismatch>,
    pub tx_wasm_length: usize,
    pub build_wasm_length: usize,
}

#[derive(Debug)]
pub struct PreludeMismatch {
    pub tx: String,
    pub build: String,
}

#[derive(Debug, thiserror::Error)]
pub enum VerificationError {
    #[error("RPC failed: {0}")]
    Rpc(#[from] alloy::transports::RpcError<alloy::transports::TransportErrorKind>),

    #[error("{0}")]
    Check(#[from] crate::core::check::CheckError),
    #[error("{0}")]
    Reflection(#[from] crate::core::reflection::ReflectionError),
    #[error("{0}")]
    Command(#[from] crate::error::CommandError),

    #[error("No code at address")]
    NoCodeAtAddress,
    #[error("Deployment transaction uses constructor but the local project doesn't have one")]
    NoConstructor,
    #[error("Invalid init data: Constructor not called")]
    InvalidInitData,
}
