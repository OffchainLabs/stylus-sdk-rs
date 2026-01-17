// Copyright 2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Deploy a Stylus contract or Stylus root contract

use alloy::{
    network::TransactionBuilder,
    primitives::{Address, U256},
    providers::Provider,
    rpc::types::{TransactionReceipt, TransactionRequest},
};

use super::{prelude::DeploymentCalldata, DeploymentError};
use crate::utils::color::DebugColor;

/// Deployment transaction request for a Stylus contract
#[derive(Debug)]
pub struct DeploymentRequest {
    tx: TransactionRequest,
    max_fee_per_gas_wei: Option<u128>,
}

impl DeploymentRequest {
    /// Deployment request for no-constructor contracts
    pub fn new(sender: Address, code: &[u8], max_fee_per_gas_wei: Option<u128>) -> Self {
        Self {
            tx: TransactionRequest::default()
                .with_from(sender)
                .with_deploy_code(DeploymentCalldata::new(code)),
            max_fee_per_gas_wei,
        }
    }

    /// Deployment request for constructor contracts
    pub fn new_with_args(
        sender: Address,
        deployer: Address,
        tx_value: U256,
        tx_calldata: Vec<u8>,
        max_fee_per_gas_wei: Option<u128>,
    ) -> Self {
        Self {
            tx: TransactionRequest::default()
                .with_to(deployer)
                .with_from(sender)
                .with_value(tx_value)
                .with_input(tx_calldata),
            max_fee_per_gas_wei,
        }
    }

    pub async fn estimate_gas(&self, provider: &impl Provider) -> Result<u64, DeploymentError> {
        Ok(provider.estimate_gas(self.tx.clone()).await?)
    }

    pub async fn exec(
        self,
        provider: &impl Provider,
    ) -> Result<TransactionReceipt, DeploymentError> {
        let gas = self.estimate_gas(provider).await?;
        let max_fee_per_gas = self.fee_per_gas(provider).await?;

        let mut tx = self.tx;
        tx.gas = Some(gas);
        tx.max_fee_per_gas = Some(max_fee_per_gas);
        tx.max_priority_fee_per_gas = Some(0);

        let tx = provider.send_transaction(tx).await?;
        let tx_hash = *tx.tx_hash();
        debug!(@grey, "sent deploy tx: {}", tx_hash.debug_lavender());

        let receipt = tx
            .get_receipt()
            .await
            .or(Err(DeploymentError::FailedToComplete))?;
        if !receipt.status() {
            return Err(DeploymentError::Reverted { tx_hash });
        }

        Ok(receipt)
    }

    async fn fee_per_gas(&self, provider: &impl Provider) -> Result<u128, DeploymentError> {
        match self.max_fee_per_gas_wei {
            Some(wei) => Ok(wei),
            None => Ok(provider.get_gas_price().await?),
        }
    }
}
