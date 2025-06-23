// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::{
    primitives::{B256, U256},
    providers::Provider,
};

use crate::precompiles::{self, ArbWasm::ArbWasmErrors};

/// Checks whether a contract has already been activated with the most recent version of Stylus.
pub async fn contract_exists(
    codehash: B256,
    provider: &impl Provider,
) -> Result<bool, ContractExistsError> {
    let arbwasm = precompiles::arb_wasm(provider);
    match arbwasm.codehashVersion(codehash).call().await {
        Ok(_) => Ok(true),
        Err(e) => {
            let alloy::contract::Error::TransportError(tperr) = e else {
                return Err(ContractExistsError::FailedToSendTx(e));
            };
            let Some(err_resp) = tperr.as_error_resp() else {
                return Err(ContractExistsError::NoErrorPayload(tperr));
            };
            let Some(errs) = err_resp.as_decoded_interface_error::<ArbWasmErrors>() else {
                return Err(ContractExistsError::FailedToDecode(err_resp.clone()));
            };
            use ArbWasmErrors as A;
            match errs {
                A::ProgramNotActivated(_) | A::ProgramNeedsUpgrade(_) | A::ProgramExpired(_) => {
                    Ok(false)
                }
                _ => Err(ContractExistsError::UnexpectedArbWasmError),
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ContractExistsError {
    #[error("failed to send tx: {0:?}")]
    FailedToSendTx(alloy::contract::Error),
    #[error("no error payload found in response: {0:?}")]
    NoErrorPayload(alloy::transports::RpcError<alloy::transports::TransportErrorKind>),
    #[error("failed to decode error: {0:?}")]
    FailedToDecode(alloy::rpc::json_rpc::ErrorPayload),
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
