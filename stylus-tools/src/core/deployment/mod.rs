// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::{
    json_abi::StateMutability::Payable,
    primitives::B256,
    primitives::{Address, TxHash, U256},
    providers::{Provider, WalletProvider},
};

use crate::{
    core::{
        activation::{self, ActivationError},
        cache::format_gas,
        check::{check_contract, CheckConfig},
        code::Code,
        deployment::{
            deployer::{get_address_from_receipt, parse_tx_calldata, DeployerError},
            DeploymentError::{InvalidConstructor, NoContractAddress, ReadConstructorFailure},
        },
        project::contract::{Contract, ContractStatus},
    },
    ops::{activate::print_gas_estimate, get_constructor_signature},
    precompiles,
    utils::color::{Color, DebugColor},
};
use fragments::{deploy_fragments, estimate_fragments_gas, estimate_root_contract_gas};
use request::DeploymentRequest;

pub mod deployer;
pub mod fragments;
pub mod prelude;
pub mod request;

/// Estimate gas cost for contract deployment
pub async fn estimate_gas(
    contract: &Contract,
    config: &DeploymentConfig,
    provider: &(impl Provider + WalletProvider),
) -> Result<u64, DeploymentError> {
    let status = check_contract(contract, None, &config.check, provider).await?;
    let from_address = provider.default_signer_address();
    debug!(@grey, "sender address: {}", from_address.debug_lavender());

    let mut gas = match status.code() {
        Code::Contract(contract) => {
            let req =
                DeploymentRequest::new(from_address, contract.bytes(), config.max_fee_per_gas_gwei);
            req.estimate_gas(provider).await?
        }
        Code::Fragments(fragments) => {
            estimate_fragments_gas(fragments, config, provider).await?
                + estimate_root_contract_gas(fragments, status.wasm().len(), config, provider)
                    .await?
        }
    };
    gas += status.suggest_fee().to::<u64>() + config.constructor_value.to::<u64>();

    let gas_price = match config.max_fee_per_gas_gwei {
        Some(gwei) => gwei,
        None => provider
            .get_gas_price()
            .await
            .or(Err(DeployerError::GasEstimationFailure))?,
    };
    print_gas_estimate("deployment", gas, gas_price)
        .or(Err(DeployerError::GasEstimationFailure))?;
    let nonce = provider.get_transaction_count(from_address).await?;
    let _ = from_address.create(nonce);
    Ok(gas)
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

    if let Code::Fragments(fragments) = status.code() {
        let arb_owner_public = precompiles::arb_owner_public(provider);
        let max_fragment_count = arb_owner_public
            .getMaxStylusContractFragments()
            .call()
            .await
            // Failing this call likely means the chain does not support fragments (old ArbOS)
            .map_err(|_| DeploymentError::ContractTooLarge)?;
        if fragments.fragment_count() > max_fragment_count as usize {
            return Err(DeploymentError::ContractTooLarge);
        }
    }

    let constructor = get_constructor_signature(contract.package.name.as_str())
        .map_err(|_| ReadConstructorFailure)?;

    let req = match &constructor {
        None => match status.code() {
            Code::Contract(contract) => {
                DeploymentRequest::new(from_address, contract.bytes(), config.max_fee_per_gas_gwei)
            }
            Code::Fragments(fragments) => {
                let root =
                    deploy_fragments(fragments, status.wasm().len(), config, provider).await?;
                DeploymentRequest::new(from_address, root.bytes(), config.max_fee_per_gas_gwei)
            }
        },
        Some(constructor) => {
            if constructor.state_mutability != Payable && !config.constructor_value.is_zero() {
                return Err(InvalidConstructor(
                    "attempting to send Ether to non-payable constructor".to_string(),
                ));
            }
            if config.constructor_args.len() != constructor.inputs.len() {
                return Err(InvalidConstructor(format!(
                    "mismatch number of constructor arguments (want {:?} ({}); got {})",
                    constructor.inputs,
                    constructor.inputs.len(),
                    config.constructor_args.len(),
                )));
            }

            let code = match status.code() {
                Code::Contract(contract) => contract.0.clone(),
                Code::Fragments(fragments) => {
                    deploy_fragments(fragments, status.wasm().len(), config, provider)
                        .await?
                        .0
                }
            };
            let tx_calldata = parse_tx_calldata(
                &code,
                constructor,
                config.constructor_value,
                config.constructor_args.clone(),
                config.deployer_salt,
                &provider,
            )
            .await
            .map_err(|err| InvalidConstructor(err.to_string()))?;

            DeploymentRequest::new_with_args(
                from_address,
                config.deployer_address,
                data_fee,
                tx_calldata,
                config.max_fee_per_gas_gwei,
            )
        }
    };

    let receipt = req.exec(&provider).await?;

    let contract_addr = match &constructor {
        None => receipt
            .contract_address
            .ok_or(NoContractAddress("in receipt".to_string())),
        Some(_) => get_address_from_receipt(&receipt),
    }?;

    info!(@grey, "deployed code at address: {}", contract_addr.debug_lavender());
    debug!(@grey, "gas used: {}", format_gas(receipt.gas_used.into()));
    info!(@grey, "deployment tx hash: {}", receipt.transaction_hash.debug_lavender());

    if constructor.is_none() {
        if matches!(status, ContractStatus::Active { .. }) {
            greyln!("wasm already activated!")
        } else if config.no_activate {
            mintln!(
                r#"NOTE:
            You must activate the stylus contract before calling it. To do so, we recommend running:
            cargo stylus activate --address {}"#,
                hex::encode(contract_addr)
            )
        } else {
            activation::activate_contract(contract_addr, &config.check.activation, provider)
                .await?;
        }
    }

    mintln!(
        r#"NOTE:
        We recommend running cargo stylus cache bid {} 0 to cache your activated contract in ArbOS.
        Cached contracts benefit from cheaper calls.
        To read more about the Stylus contract cache, see:
        https://docs.arbitrum.io/stylus/how-tos/caching-contracts"#,
        hex::encode(contract_addr)
    );

    Ok(())
}

#[derive(Debug, Default)]
pub struct DeploymentConfig {
    pub check: CheckConfig,
    pub max_fee_per_gas_gwei: Option<u128>,
    pub no_activate: bool,
    pub constructor_value: U256,
    pub deployer_address: Address,
    pub constructor_args: Vec<String>,
    pub deployer_salt: B256,
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
    #[error("missing address: {0}")]
    NoContractAddress(String),
    #[error("failed to get constructor signature")]
    ReadConstructorFailure,
    #[error("invalid constructor: {0}")]
    InvalidConstructor(String),
    #[error("contract too large, cannot deploy")]
    ContractTooLarge,
}
