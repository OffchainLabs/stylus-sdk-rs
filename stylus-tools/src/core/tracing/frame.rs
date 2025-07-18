// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::mem;

use alloy::primitives::{Address, B256, U256};
use serde::Deserialize;
use serde_json::Value;

use super::{hostio::Hostio, TracingError};

//#[derive(Serialize, Deserialize)]
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ActivationTraceFrame {
    address: Value,
}

//#[derive(Clone, Debug, PartialEq, Eq)]
#[derive(Debug)]
#[allow(dead_code)]
pub struct TraceFrame {
    steps: Vec<Hostio>,
    address: Option<Address>,
}

impl TraceFrame {
    pub fn new(address: Option<Address>) -> Self {
        let steps = Vec::new();
        Self { steps, address }
    }

    pub fn parse_frame(address: Option<Address>, value: Value) -> Result<TraceFrame, TracingError> {
        let mut frame = TraceFrame::new(address);

        let Value::Array(array) = value else {
            return Err(TracingError::NotAnArray { value });
        };

        for step in array {
            let Value::Object(mut keys) = step else {
                return Err(TracingError::InvalidStep { value: step });
            };

            macro_rules! get_typed {
                ($keys:expr, $ty:ident, $name:expr) => {{
                    match $keys.remove($name) {
                        Some(Value::$ty(value)) => value,
                        Some(value) => {
                            return Err(TracingError::UnexpectedType { key: $name, value })
                        }
                        None => {
                            return Err(TracingError::ObjectMissingKey {
                                key: $name,
                                items: $keys,
                            })
                        }
                    }
                }};
            }
            macro_rules! get_int {
                ($name:expr) => {
                    get_typed!(keys, Number, $name).as_u64().unwrap()
                };
            }
            macro_rules! get_hex {
                ($name:expr) => {{
                    let data = get_typed!(keys, String, $name);
                    let data = data.strip_prefix("0x").ok_or(TracingError::NoPrefix {
                        name: stringify!($name),
                    })?;
                    hex::decode(data)
                        .map_err(|_| TracingError::ParsingFailed { name: $name })?
                        .into_boxed_slice()
                }};
            }

            let name = get_typed!(keys, String, "name");
            let mut args = get_hex!("args");
            let mut outs = get_hex!("outs");

            let start_ink = get_int!("startInk");
            let end_ink = get_int!("endInk");

            macro_rules! read_data {
                ($src:ident) => {{
                    let data = $src;
                    $src = Box::new([]);
                    data
                }};
            }
            macro_rules! read_ty {
                ($src:ident, $ty:ident, $conv:expr) => {{
                    let size = mem::size_of::<$ty>();
                    let len = $src.len();
                    if size > len {
                        return Err(TracingError::NotEnoughBytes {
                            name: stringify!($src),
                            size,
                            len,
                        });
                    }
                    let (left, right) = $src.split_at(size);
                    let result = $conv(left);
                    $src = right.to_vec().into_boxed_slice();
                    result
                }};
            }
            macro_rules! read_string {
                ($src:ident) => {{
                    let conv = |x: &[_]| String::from_utf8_lossy(&x).to_string();
                    read_ty!($src, String, conv)
                }};
            }
            macro_rules! read_u256 {
                ($src:ident) => {
                    read_ty!($src, U256, |x| B256::from_slice(x).into())
                };
            }
            macro_rules! read_b256 {
                ($src:ident) => {
                    read_ty!($src, B256, B256::from_slice)
                };
            }
            macro_rules! read_address {
                ($src:ident) => {
                    read_ty!($src, Address, Address::from_slice)
                };
            }
            macro_rules! read_num {
                ($src:ident, $ty:ident) => {{
                    let conv = |x: &[_]| $ty::from_be_bytes(x.try_into().unwrap());
                    read_ty!($src, $ty, conv)
                }};
            }
            macro_rules! read_u8 {
                ($src:ident) => {
                    read_num!($src, u8)
                };
            }
            macro_rules! read_u16 {
                ($src:ident) => {
                    read_num!($src, u16)
                };
            }
            macro_rules! read_u32 {
                ($src:ident) => {
                    read_num!($src, u32)
                };
            }
            macro_rules! read_u64 {
                ($src:ident) => {
                    read_num!($src, u64)
                };
            }

            macro_rules! frame {
                () => {{
                    let address = get_hex!("address");
                    let address = Address::from_slice(&address);
                    let steps = keys.remove("steps").unwrap();
                    TraceFrame::parse_frame(Some(address), steps)?
                }};
            }

            use super::hostio::HostioKind::*;
            let kind = match name.as_str() {
                "user_entrypoint" => UserEntrypoint {
                    args_len: read_u32!(args),
                },
                "user_returned" => UserReturned {
                    status: read_u32!(outs),
                },
                "read_args" => ReadArgs {
                    args: read_data!(outs),
                },
                "write_result" => WriteResult {
                    result: read_data!(args),
                },
                "exit_early" => ExitEarly {
                    status: read_u32!(args),
                },
                "storage_load_bytes32" => StorageLoadBytes32 {
                    key: read_b256!(args),
                    value: read_b256!(outs),
                },
                "storage_cache_bytes32" => StorageCacheBytes32 {
                    key: read_b256!(args),
                    value: read_b256!(args),
                },
                "storage_flush_cache" => StorageFlushCache {
                    clear: read_u8!(args),
                },
                "transient_load_bytes32" => TransientLoadBytes32 {
                    key: read_b256!(args),
                    value: read_b256!(outs),
                },
                "transient_store_bytes32" => TransientStoreBytes32 {
                    key: read_b256!(args),
                    value: read_b256!(args),
                },
                "account_balance" => AccountBalance {
                    address: read_address!(args),
                    balance: read_u256!(outs),
                },
                "account_code" => AccountCode {
                    address: read_address!(args),
                    offset: read_u32!(args),
                    size: read_u32!(args),
                    code: read_data!(outs),
                },
                "account_code_size" => AccountCodeSize {
                    address: read_address!(args),
                    size: read_u32!(outs),
                },
                "account_codehash" => AccountCodehash {
                    address: read_address!(args),
                    codehash: read_b256!(outs),
                },
                "block_basefee" => BlockBasefee {
                    basefee: read_u256!(outs),
                },
                "block_coinbase" => BlockCoinbase {
                    coinbase: read_address!(outs),
                },
                "block_gas_limit" => BlockGasLimit {
                    limit: read_u64!(outs),
                },
                "block_number" => BlockNumber {
                    number: read_u64!(outs),
                },
                "block_timestamp" => BlockTimestamp {
                    timestamp: read_u64!(outs),
                },
                "chainid" => Chainid {
                    chainid: read_u64!(outs),
                },
                "contract_address" => ContractAddress {
                    address: read_address!(outs),
                },
                "evm_gas_left" => EvmGasLeft {
                    gas_left: read_u64!(outs),
                },
                "evm_ink_left" => EvmInkLeft {
                    ink_left: read_u64!(outs),
                },
                "math_div" => MathDiv {
                    a: read_u256!(args),
                    b: read_u256!(args),
                    result: read_u256!(outs),
                },
                "math_mod" => MathMod {
                    a: read_u256!(args),
                    b: read_u256!(args),
                    result: read_u256!(outs),
                },
                "math_pow" => MathPow {
                    a: read_u256!(args),
                    b: read_u256!(args),
                    result: read_u256!(outs),
                },
                "math_add_mod" => MathAddMod {
                    a: read_u256!(args),
                    b: read_u256!(args),
                    c: read_u256!(args),
                    result: read_u256!(outs),
                },
                "math_mul_mod" => MathMulMod {
                    a: read_u256!(args),
                    b: read_u256!(args),
                    c: read_u256!(args),
                    result: read_u256!(outs),
                },
                "msg_reentrant" => MsgReentrant {
                    reentrant: read_u32!(outs) != 0,
                },
                "msg_sender" => MsgSender {
                    sender: read_address!(outs),
                },
                "msg_value" => MsgValue {
                    value: read_b256!(outs),
                },
                "native_keccak256" => NativeKeccak256 {
                    preimage: read_data!(args),
                    digest: read_b256!(outs),
                },
                "tx_gas_price" => TxGasPrice {
                    gas_price: read_u256!(outs),
                },
                "tx_ink_price" => TxInkPrice {
                    ink_price: read_u32!(outs),
                },
                "tx_origin" => TxOrigin {
                    origin: read_address!(outs),
                },
                "pay_for_memory_grow" => PayForMemoryGrow {
                    pages: read_u16!(args),
                },
                "call_contract" => CallContract {
                    address: read_address!(args),
                    gas: read_u64!(args),
                    value: read_u256!(args),
                    data: read_data!(args),
                    outs_len: read_u32!(outs),
                    status: read_u8!(outs),
                    frame: frame!(),
                },
                "delegate_call_contract" => DelegateCallContract {
                    address: read_address!(args),
                    gas: read_u64!(args),
                    data: read_data!(args),
                    outs_len: read_u32!(outs),
                    status: read_u8!(outs),
                    frame: frame!(),
                },
                "static_call_contract" => StaticCallContract {
                    address: read_address!(args),
                    gas: read_u64!(args),
                    data: read_data!(args),
                    outs_len: read_u32!(outs),
                    status: read_u8!(outs),
                    frame: frame!(),
                },
                "create1" => Create1 {
                    endowment: read_u256!(args),
                    code: read_data!(args),
                    address: read_address!(outs),
                    revert_data_len: read_u32!(outs),
                },
                "create2" => Create2 {
                    endowment: read_u256!(args),
                    salt: read_b256!(args),
                    code: read_data!(args),
                    address: read_address!(outs),
                    revert_data_len: read_u32!(outs),
                },
                "emit_log" => EmitLog {
                    topics: read_u32!(args),
                    data: read_data!(args),
                },
                "read_return_data" => ReadReturnData {
                    offset: read_u32!(args),
                    size: read_u32!(args),
                    data: read_data!(outs),
                },
                "return_data_size" => ReturnDataSize {
                    size: read_u32!(outs),
                },
                "console_log_text" => ConsoleLogText {
                    text: read_data!(args),
                },
                "console_log" => ConsoleLog {
                    text: read_string!(args),
                },
                x => {
                    if x.starts_with("evm_") {
                        EVMCall {
                            name: x.to_owned(),
                            frame: frame!(),
                        }
                    } else {
                        todo!("Missing hostio details {x}")
                    }
                }
            };

            assert!(args.is_empty(), "{name}");
            assert!(outs.is_empty(), "{name}");

            frame.steps.push(Hostio {
                kind,
                start_ink,
                end_ink,
            });
        }
        Ok(frame)
    }
}
