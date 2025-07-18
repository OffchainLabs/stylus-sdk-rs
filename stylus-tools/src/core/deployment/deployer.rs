// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::{
    primitives::{address, Address, U256},
    providers::Provider,
    rpc::types::TransactionRequest,
    sol,
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

#[derive(Debug)]
pub struct DeployerArgs {
    /// Factory address
    address: Address,
    /// Value to be sent in the tx
    tx_value: U256,
    /// Calldata to be sent in the tx
    tx_calldata: Vec<u8>,
}

/// Deploys, activates, and initializes the contract using the Stylus deployer.
pub async fn deploy(
    deployer: DeployerArgs,
    sender: Address,
    provider: &impl Provider,
) -> Result<(), DeployerError> {
    debug!(@grey, "deploying contract using deployer at address: {}", deployer.address);
    let tx = TransactionRequest::default()
        .to(deployer.address)
        .from(sender)
        .value(deployer.tx_value)
        .input(deployer.tx_calldata.into());

    let _gas = provider
        .estimate_gas(tx.clone())
        .await
        .or(Err(DeployerError::GasEstimationFailure))?;

    let _gas_price = provider.get_gas_price().await?;
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum DeployerError {
    #[error("rpc error: {0}")]
    Rpc(#[from] alloy::transports::RpcError<alloy::transports::TransportErrorKind>),

    #[error("deployment failed during gas estimation")]
    GasEstimationFailure,
}
