// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::{
    network::TransactionBuilder,
    primitives::{Address, TxHash, U256},
    providers::{Provider, WalletProvider},
    rpc::types::{TransactionReceipt, TransactionRequest},
};

use crate::{
    core::{
        check::{check_contract, CheckConfig},
        project::contract::{Contract, ContractStatus},
    },
    utils::color::{Color, DebugColor},
};
use prelude::DeploymentCalldata;

pub mod deployer;
pub mod prelude;

#[derive(Debug, Default)]
pub struct DeploymentConfig {
    pub check: CheckConfig,
    pub constructor_value: U256,
}

#[derive(Debug)]
pub struct DeploymentRequest {
    tx: TransactionRequest,
    max_fee_per_gas_wei: Option<u128>,
}

impl DeploymentRequest {
    pub fn new(sender: Address, code: &[u8]) -> Self {
        let deploy_code = DeploymentCalldata::new(code);
        let tx = TransactionRequest::default()
            .with_from(sender)
            .with_deploy_code(deploy_code);
        Self {
            tx,
            max_fee_per_gas_wei: None,
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

#[derive(Debug, thiserror::Error)]
pub enum DeploymentError {
    #[error("rpc error: {0}")]
    Rpc(#[from] alloy::transports::RpcError<alloy::transports::TransportErrorKind>),

    #[error("{0}")]
    Check(#[from] crate::core::check::CheckError),

    #[error("tx failed to complete")]
    FailedToComplete,
    #[error("failed to get balance")]
    FailedToGetBalance,
    #[error(
        "not enough funds in account {} to pay for data fee\n\
         balance {} < {}\n\
         please see the Quickstart guide for funding new accounts:\n{}",
        .from_address.red(),
        .balance.red(),
        format!("{} wei", .data_fee).red(),
        "https://docs.arbitrum.io/stylus/stylus-quickstart".yellow(),
    )]
    NotEnoughFunds {
        from_address: Address,
        balance: U256,
        data_fee: U256,
    },
    #[error("deploy tx reverted {}", .tx_hash.debug_red())]
    Reverted { tx_hash: TxHash },
}

/// Deploys a stylus contract, activating if needed.
pub async fn deploy(
    contract: &Contract,
    config: &DeploymentConfig,
    provider: &(impl Provider + WalletProvider),
) -> Result<(), DeploymentError> {
    let status = check_contract(contract, None, &config.check, provider).await?;
    let from_address = provider.default_signer_address();
    debug!(@grey, "sender address: {}", from_address.debug_lavender());
    let data_fee = status.suggest_fee() + config.constructor_value;

    if let ContractStatus::Ready { .. } = status {
        // check balance early
        let balance = provider
            .get_balance(from_address)
            .await
            .map_err(|_| DeploymentError::FailedToGetBalance)?;
        if balance < data_fee {
            return Err(DeploymentError::NotEnoughFunds {
                from_address,
                balance,
                data_fee,
            });
        }
    }

    Ok(())
}
