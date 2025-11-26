// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::collections::VecDeque;

use alloy::{
    eips::BlockId,
    network::TransactionBuilder,
    primitives::{Address, Bytes, TxHash, U256},
    providers::{ext::DebugApi, Provider},
    rpc::types::{
        trace::geth::{
            GethDebugTracerType, GethDebugTracingCallOptions, GethDebugTracingOptions, GethTrace,
        },
        TransactionRequest,
    },
};
use eyre::bail;
use serde_json::{Map, Value};

use frame::{ActivationTraceFrame, TraceFrame};

use crate::{
    core::tracing::hostio::Hostio,
    utils::color::{Color, DebugColor},
};

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
        config: &TraceConfig,
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

        let query = config.query();

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

    pub fn reader(self) -> FrameReader {
        FrameReader {
            steps: self.top_frame.steps.clone().into(),
            frame: self.top_frame,
        }
    }

    pub async fn simulate(
        config: &SimulateConfig,
        provider: &impl Provider,
    ) -> Result<Self, TracingError> {
        let tx_request = config.build_transaction_request();
        let query = config.trace.query();

        // Corrected construction of tracer_options
        let tracer_options = GethDebugTracingCallOptions {
            tracing_options: GethDebugTracingOptions {
                tracer: Some(GethDebugTracerType::JsTracer(query.to_string())),
                ..Default::default()
            },
            ..Default::default()
        };

        // Use the latest block; alternatively, this can be made configurable
        let block_id = BlockId::latest();

        let GethTrace::JS(json) = provider
            .debug_trace_call(tx_request.clone(), block_id, tracer_options)
            .await?
        else {
            return Err(TracingError::MalformedResult);
        };

        if let Value::Array(arr) = json.clone() {
            if arr.is_empty() {
                return Err(TracingError::NoFrames);
            }
        }

        // Parse the trace frames
        let top_frame = TraceFrame::parse_frame(None, json.clone())?;

        Ok(Self {
            top_frame,
            tx: tx_request,
            json,
        })
    }

    pub fn json(&self) -> &Value {
        &self.json
    }

    pub fn tx(&self) -> &TransactionRequest {
        &self.tx
    }

    pub fn address(&self) -> Option<Address> {
        self.top_frame.address
    }
}

#[derive(Debug)]
pub struct FrameReader {
    frame: TraceFrame,
    steps: VecDeque<Hostio>,
}

impl FrameReader {
    fn next(&mut self) -> eyre::Result<Hostio> {
        match self.steps.pop_front() {
            Some(item) => Ok(item),
            None => bail!("No next hostio"),
        }
    }

    pub fn next_hostio(&mut self, expected: &'static str) -> Hostio {
        fn detected(reader: &FrameReader, expected: &'static str) {
            let expected = expected.red();
            let which = match reader.frame.address {
                Some(call) => format!("call to {}", call.red()),
                None => "contract deployment".to_string(),
            };
            println!("{}", "\n════════ Divergence ════════".red());
            println!("Divegence detected while simulating a {which} via local assembly.");
            println!("The simulated environment expected a call to the {expected} Host I/O.",);
        }

        loop {
            let Ok(hostio) = self.next() else {
                detected(self, expected);
                println!("However, no such call is made onchain. Are you sure this the right contract?\n");
                panic!();
            };

            if hostio.kind.name() == expected {
                return hostio;
            }

            let kind = hostio.kind;
            let name = kind.name();
            match name {
                "pay_for_memory_grow" | "user_entrypoint" | "user_returned" => continue,
                _ => {
                    detected(self, expected);
                    println!("However, onchain there's a call to {name}. Are you sure this the right contract?\n");
                    println!("expected: {}", expected.red());
                    println!("but have: {}\n", kind.debug_red());
                    panic!();
                }
            }
        }
    }
}

#[derive(Debug, clap::Args)]
pub struct TraceConfig {
    /// If set, use the native tracer instead of the JavaScript one.
    #[arg(short, long, default_value_t = false)]
    use_native_tracer: bool,
}

impl TraceConfig {
    fn query(&self) -> &'static str {
        if self.use_native_tracer {
            "stylusTracer"
        } else {
            include_str!("query.js")
        }
    }
}

#[derive(Debug, clap::Args)]
pub struct SimulateConfig {
    /// From address.
    #[arg(short, long)]
    from: Option<Address>,

    /// To address.
    #[arg(short, long)]
    to: Option<Address>,

    /// Gas limit.
    #[arg(long)]
    gas: Option<u64>,

    /// Gas price.
    #[arg(long)]
    gas_price: Option<u128>,

    /// Value to send with the transaction.
    #[arg(short, long)]
    value: Option<U256>,

    /// Data to send with the transaction, as a hex string (with or without '0x' prefix).
    #[arg(short, long)]
    data: Option<Bytes>,

    #[command(flatten)]
    trace: TraceConfig,
}

impl SimulateConfig {
    fn build_transaction_request(&self) -> TransactionRequest {
        let mut tx_request = TransactionRequest::default();

        if let Some(from) = self.from {
            tx_request = tx_request.with_from(from);
        }
        if let Some(to) = self.to {
            tx_request = tx_request.with_to(to);
        }
        if let Some(gas) = self.gas {
            tx_request = tx_request.with_gas_limit(gas);
        }
        if let Some(gas_price) = self.gas_price {
            tx_request = tx_request.with_max_fee_per_gas(gas_price);
        }
        if let Some(value) = self.value {
            tx_request = tx_request.with_value(value);
        }
        if let Some(data) = &self.data {
            tx_request = tx_request.with_input(data.clone());
        }

        tx_request
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
