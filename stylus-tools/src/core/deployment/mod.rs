// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use crate::core::activation;
use crate::core::activation::ActivationError;
use crate::core::cache::format_gas;
use crate::core::deployment::deployer::{DeployerArgs, DeployerError};
use crate::core::deployment::DeploymentError::NoContractAddress;
use crate::ops::activate::print_gas_estimate;
use crate::{
    core::{
        check::{check_contract, CheckConfig},
        project::contract::{Contract, ContractStatus},
    },
    utils::color::{Color, DebugColor},
};
use alloy::{
    network::TransactionBuilder,
    primitives::{Address, TxHash, U256},
    providers::{Provider, WalletProvider},
    rpc::types::{TransactionReceipt, TransactionRequest},
};
use prelude::DeploymentCalldata;

pub mod deployer;
pub mod prelude;

#[derive(Debug, Default)]
pub struct DeploymentConfig {
    pub check: CheckConfig,
    pub max_fee_per_gas_gwei: Option<u128>,
    pub estimate_gas: bool,
    pub no_activate: bool,
    pub constructor_value: U256,
}

#[derive(Debug)]
pub struct DeploymentRequest {
    tx: TransactionRequest,
    max_fee_per_gas_wei: Option<u128>,
}

impl DeploymentRequest {
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
    pub fn new(sender: Address, code: &[u8], max_fee_per_gas_wei: Option<u128>) -> Self {
        Self {
            tx: TransactionRequest::default()
                .with_from(sender)
                .with_deploy_code(DeploymentCalldata::new(code)),
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

        println!("Sending tx: {:?}", tx);
        let tx = provider.send_transaction(tx).await?;
        let tx_hash = *tx.tx_hash();
        println!("Sent tx: {:?}", tx);
        debug!(@grey, "sent deploy tx: {}", tx_hash.debug_lavender());

        let receipt = tx
            .get_receipt()
            .await
            .or(Err(DeploymentError::FailedToComplete))?;
        if !receipt.status() {
            return Err(DeploymentError::Reverted { tx_hash });
        }

        println!("Received receipt: {:?}", receipt);
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
    #[error("{0}")]
    DeployerFailure(#[from] DeployerError),
    #[error("{0}")]
    ActivationFailure(#[from] ActivationError),
    // TODO: Can this error occur?
    #[error("missing address")]
    NoContractAddress,
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

    // TODO: Branch for arg constructor
    let req = DeploymentRequest::new(from_address, status.code(), config.max_fee_per_gas_gwei);

    if config.estimate_gas {
        let gas = req
            .estimate_gas(&provider)
            .await
            .or(Err(DeployerError::GasEstimationFailure))?;
        let gas_price = req
            .fee_per_gas(&provider)
            .await
            .or(Err(DeployerError::GasEstimationFailure))?;
        print_gas_estimate("deployment", gas, gas_price)
            .or(Err(DeployerError::GasEstimationFailure))?;
        // TODO: Is this part needed?
        let nonce = provider.get_transaction_count(from_address).await?;
        println!("Estimating {gas} {gas_price} {nonce}");
        let _ = from_address.create(nonce);
        return Ok(());
    }
    let receipt = req.exec(&provider).await?;

    // TODO: Branch for arg constructor
    let contract_addr =  receipt.contract_address.ok_or(NoContractAddress)?;

    info!(@grey, "deployed code at address: {}", contract_addr.debug_lavender());
    debug!(@grey, "gas used: {}", format_gas(receipt.gas_used.into()));
    info!(@grey, "deployment tx hash: {}", receipt.transaction_hash.debug_lavender());

    // TODO: Branch for arg constructor
    match (status, config.no_activate) {
        (ContractStatus::Ready { .. }, true) => mintln!(
            r#"NOTE: You must activate the stylus contract before calling it. To do so, we recommend running:
cargo stylus activate --address {}"#,
            hex::encode(contract_addr)
        ),
        (ContractStatus::Ready { .. }, false) => {
            activation::activate_contract(contract_addr, &config.check.activation, provider)
                .await?;
        }
        (ContractStatus::Active { .. }, _) => greyln!("wasm already activated!"),
    }
    print_cache_notice(contract_addr);

    Ok(())
}

pub fn print_cache_notice(contract_addr: Address) {
    let contract_addr = hex::encode(contract_addr);
    mintln!(
        r#"NOTE: We recommend running cargo stylus cache bid {contract_addr} 0 to cache your activated contract in ArbOS.
Cached contracts benefit from cheaper calls. To read more about the Stylus contract cache, see
https://docs.arbitrum.io/stylus/how-tos/caching-contracts"#
    );
}
