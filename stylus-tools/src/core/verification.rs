// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::{
    consensus::Transaction,
    primitives::{Address, TxHash},
    providers::Provider,
    sol_types::SolCall,
};

use crate::{
    core::{
        code::Code,
        deployment::{
            deployer::{stylus_constructorCall, StylusDeployer::deployCall, ADDRESS},
            prelude::DeploymentCalldata,
        },
        project::contract::Contract,
        reflection,
        verification::VerificationError::{
            InvalidDeployerAddress, InvalidInitData, TransactionReceiptError, TxNotSuccessful,
        },
    },
    utils::cargo,
};

pub async fn verify(
    contract: &Contract,
    tx_hash: TxHash,
    skip_clean: bool,
    provider: &impl Provider,
) -> Result<VerificationStatus, VerificationError> {
    let tx = provider
        .get_transaction_by_hash(tx_hash)
        .await?
        .ok_or(VerificationError::NoCodeAtAddress)?;
    if !skip_clean {
        cargo::clean()?;
    }
    let deployment_success = provider
        .get_transaction_receipt(tx_hash)
        .await?
        .map(|receipt| receipt.status())
        .ok_or(TransactionReceiptError)?;
    if !deployment_success {
        return Err(TxNotSuccessful);
    }
    let status = contract.check(None, &Default::default(), provider).await?;
    let deployment_data = match status.code() {
        Code::Contract(contract) => DeploymentCalldata::new(contract.bytes()),
        Code::Fragments(_fragments) => todo!("support fragments for verification"),
    };

    match tx.to() {
        Some(deployer_address) => {
            verify_constructor_deployment(tx.input(), &deployment_data, deployer_address)
        }
        _ => verify_create_deployment(&DeploymentCalldata(tx.input().to_vec()), &deployment_data),
    }
}

fn verify_constructor_deployment(
    tx_input: &[u8],
    deployment_data: &DeploymentCalldata,
    deployer_address: Address,
) -> Result<VerificationStatus, VerificationError> {
    let _constructor = reflection::constructor()?.ok_or(VerificationError::NoConstructor)?;
    let deploy_call = deployCall::abi_decode(tx_input).unwrap();
    let constructor_called = deploy_call
        .initData
        .starts_with(stylus_constructorCall::SELECTOR.as_slice());
    if !constructor_called {
        return Err(InvalidInitData);
    }
    if deployer_address != ADDRESS {
        return Err(InvalidDeployerAddress);
    }
    verify_create_deployment(
        &DeploymentCalldata(deploy_call.bytecode.to_vec()),
        deployment_data,
    )
}

fn verify_create_deployment(
    calldata: &DeploymentCalldata,
    deployment_data: &DeploymentCalldata,
) -> Result<VerificationStatus, VerificationError> {
    if deployment_data == calldata {
        return Ok(VerificationStatus::Success);
    }

    let tx_prelude = calldata.prelude();
    let build_prelude = deployment_data.prelude();
    let prelude_mismatch = if tx_prelude == build_prelude {
        None
    } else {
        Some(PreludeMismatch {
            tx: hex::encode(tx_prelude),
            build: hex::encode(build_prelude),
        })
    };

    let tx_wasm_length = calldata.compressed_wasm().len();
    let build_wasm_length = deployment_data.compressed_wasm().len();
    Ok(VerificationStatus::Failure(VerificationFailure {
        prelude_mismatch,
        tx_wasm_length,
        build_wasm_length,
    }))
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
    #[error("Invalid deployer address")]
    InvalidDeployerAddress,
    #[error("Transaction receipt error")]
    TransactionReceiptError,
    #[error("Deployment transaction not successful")]
    TxNotSuccessful,
}
