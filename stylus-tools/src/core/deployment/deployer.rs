// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::{
    dyn_abi::{DynSolValue, JsonAbiExt, Specifier},
    json_abi::Constructor,
    primitives::{address, Address, B256, U256},
    providers::Provider,
    rpc::types::TransactionReceipt,
    sol,
    sol_types::{SolCall, SolEvent},
};
use eyre::{Context, ErrReport};

use crate::core::deployment::{
    prelude::DeploymentCalldata, DeploymentError, DeploymentError::NoContractAddress,
};

pub const ADDRESS: Address = address!("cEcba2F1DC234f70Dd89F2041029807F8D03A990");

sol! {
    #[sol(rpc)]
    interface StylusDeployer {
        event ContractDeployed(address deployedContract);

        function deploy(
            bytes calldata bytecode,
            bytes calldata initData,
            uint256 initValue,
            bytes32 salt
        ) public payable returns (address);
    }

    function stylus_constructor();
}

#[derive(Debug, thiserror::Error)]
pub enum DeployerError {
    #[error("rpc error: {0}")]
    Rpc(#[from] alloy::transports::RpcError<alloy::transports::TransportErrorKind>),

    #[error("deployment failed during gas estimation")]
    GasEstimationFailure,
}

pub async fn parse_tx_calldata(
    contract_code: &[u8],
    constructor: &Constructor,
    constructor_value: U256,
    constructor_args: Vec<String>,
    deployer_salt: B256,
    provider: &impl Provider,
) -> Result<Vec<u8>, ErrReport> {
    let mut arg_values = Vec::<DynSolValue>::with_capacity(constructor_args.len());
    for (arg, param) in constructor_args.iter().zip(constructor.inputs.iter()) {
        let ty = param
            .resolve()
            .wrap_err_with(|| format!("could not resolve constructor arg: {param}"))?;
        let value = ty
            .coerce_str(arg)
            .wrap_err_with(|| format!("could not parse constructor arg: {param}"))?;
        arg_values.push(value);
    }

    let calldata_args = constructor.abi_encode_input_raw(&arg_values)?;

    let mut constructor_calldata = Vec::from(stylus_constructorCall::SELECTOR);
    constructor_calldata.extend(calldata_args);

    let tx_calldata = StylusDeployer::new(Address::ZERO, provider)
        .deploy_call(
            DeploymentCalldata::new(contract_code).into(),
            constructor_calldata.into(),
            constructor_value,
            deployer_salt,
        )
        .calldata()
        .to_vec();
    Ok(tx_calldata)
}

/// Gets the Stylus-contract address that was deployed using the deployer.
pub fn get_address_from_receipt(receipt: &TransactionReceipt) -> Result<Address, DeploymentError> {
    receipt
        .clone()
        .into_inner()
        .logs()
        .iter()
        .find(|log| match log.topics().first() {
            Some(topic) => topic.0 == StylusDeployer::ContractDeployed::SIGNATURE_HASH,
            None => false,
        })
        .map(|log| {
            if log.data().data.len() != 32 {
                return Err(NoContractAddress("from ContractDeployed log".to_string()));
            }
            Ok(Address::from_slice(&log.data().data[12..32]))
        })
        .unwrap_or_else(|| Err(NoContractAddress("from receipt logs".to_string())))
}
