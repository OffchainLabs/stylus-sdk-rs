// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::{
    primitives::TxHash,
    providers::{ext::DebugApi, Provider},
    rpc::types::{
        trace::geth::{GethDebugTracerType, GethDebugTracingOptions, GethTrace},
        TransactionRequest,
    },
};

use serde_json::{Map, Value};

use frame::{ActivationTraceFrame, TraceFrame};

pub mod frame;
pub mod hostio;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Trace {
    top_frame: TraceFrame,
    tx: TransactionRequest,
    json: Value,
}

impl Trace {
    pub async fn new(
        tx_hash: TxHash,
        use_native_tracer: bool,
        provider: &impl Provider,
    ) -> Result<Self, TracingError> {
        let receipt = provider
            .get_transaction_receipt(tx_hash)
            .await?
            .ok_or(TracingError::NoTxReceipt { tx_hash })?;
        let tx = provider
            .get_transaction_by_hash(tx_hash)
            .await?
            .ok_or(TracingError::NoTxData { tx_hash })?;

        let query = if use_native_tracer {
            "stylusTracer"
        } else {
            include_str!("query.js")
        };

        let tracer = GethDebugTracingOptions {
            tracer: Some(GethDebugTracerType::JsTracer(query.to_string())),
            ..Default::default()
        };
        let GethTrace::JS(json) = provider.debug_trace_transaction(tx_hash, tracer).await? else {
            return Err(TracingError::MalformedResult);
        };
        if let Value::Array(arr) = &json {
            if arr.is_empty() {
                return Err(TracingError::NoFrames);
            }
        }

        let maybe_activation_trace: Result<Vec<ActivationTraceFrame>, _> =
            serde_json::from_value(json.clone());
        if maybe_activation_trace.is_ok() {
            return Err(TracingError::ContractActivation);
        }

        let top_frame = TraceFrame::parse_frame(receipt.to, json.clone())?;
        Ok(Self {
            top_frame,
            tx: tx.into_request(),
            json,
        })
    }

    pub fn json(&self) -> &Value {
        &self.json
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TracingError {
    #[error("rpc error: {0}")]
    Rpc(#[from] alloy::transports::RpcError<alloy::transports::TransportErrorKind>),

    #[error("Your tx was a contract activation transaction. It has no trace frames")]
    ContractActivation,
    #[error("not a valid step: {value}")]
    InvalidStep { value: Value },
    #[error("malformed tracing result")]
    MalformedResult,
    #[error("No trace frames found, perhaps you are attempting to trace the contract deployment transaction")]
    NoFrames,
    #[error("{name} does not contain 0x prefix")]
    NoPrefix { name: &'static str },
    #[error("failed to get receipt for tx: {tx_hash}")]
    NoTxReceipt { tx_hash: TxHash },
    #[error("failed to get tx data: {tx_hash}")]
    NoTxData { tx_hash: TxHash },
    #[error("not an array: {value}")]
    NotAnArray { value: Value },
    #[error("parse {name}: want {size} bytes; got {len}")]
    NotEnoughBytes {
        name: &'static str,
        size: usize,
        len: usize,
    },
    #[error("object missing {key}: {items:?}")]
    ObjectMissingKey {
        key: &'static str,
        items: Map<String, Value>,
    },
    #[error("failed to parse {name}")]
    ParsingFailed { name: &'static str },
    #[error("unexpected type for {key}: {value}")]
    UnexpectedType { key: &'static str, value: Value },
}
