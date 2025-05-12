// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::sol_types::SolInterface;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("from utf8 error: {0}")]
    FromUtf8(#[from] std::string::FromUtf8Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("rpc error: {0}")]
    Rpc(#[from] alloy::transports::RpcError<alloy::transports::TransportErrorKind>),
    #[error("cargo metadata error: {0}")]
    CargoMetadata(#[from] cargo_metadata::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("toml serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
    #[error("toml deserialize error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),

    // TODO: better error formatting
    #[error("command failed (exit code: {code:?})", code = .0.exit_code)]
    CommandFailure(crate::core::message::ProcessOutput),
    #[error("{0}")]
    Build(#[from] crate::core::build::BuildError),
    #[error("{0}")]
    Toolchain(#[from] crate::utils::toolchain::ToolchainError),
}

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    CommandFailure(#[from] CommandFailure),
}

#[derive(Debug, thiserror::Error)]
#[error("command failed (exit code: {code:?}", code = .0.exit_code)]
pub struct CommandFailure(crate::core::message::ProcessOutput);

impl CommandFailure {
    pub fn check(
        process_name: impl Into<String>,
        output: std::process::Output,
    ) -> Result<String, Self> {
        let process_output = crate::core::message::ProcessOutput {
            process_name: process_name.into(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code(),
        };
        if output.status.success() {
            Ok(process_output.stdout)
        } else {
            Err(CommandFailure(process_output))
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ContractDecodeError {
    #[error("failed to send tx: {0:?}")]
    FailedToSendTx(alloy::contract::Error),
    #[error("no error payload found in response: {0:?}")]
    NoErrorPayload(alloy::transports::RpcError<alloy::transports::TransportErrorKind>),
    #[error("failed to decode error: {0:?}")]
    FailedToDecode(alloy::rpc::json_rpc::ErrorPayload),
}

pub fn decode_contract_error<E: SolInterface>(
    e: alloy::contract::Error,
) -> Result<E, ContractDecodeError> {
    let alloy::contract::Error::TransportError(tperr) = e else {
        return Err(ContractDecodeError::FailedToSendTx(e));
    };
    let Some(err_resp) = tperr.as_error_resp() else {
        return Err(ContractDecodeError::NoErrorPayload(tperr));
    };
    let Some(errs) = err_resp.as_decoded_interface_error::<E>() else {
        return Err(ContractDecodeError::FailedToDecode(err_resp.clone()));
    };
    Ok(errs)
}
