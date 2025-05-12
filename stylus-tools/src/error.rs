// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::transports::{RpcError, TransportErrorKind};
use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("toml serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
    #[error("toml deserialize error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),
    #[error("rpc error: {0}")]
    Rpc(#[from] RpcError<TransportErrorKind>),
}
