// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::primitives::{Address, B256, U256};

use super::frame::TraceFrame;

//#[derive(Clone, Debug, Eq, PartialEq)]
#[derive(Debug)]
pub struct Hostio {
    pub kind: HostioKind,
    pub start_ink: u64,
    pub end_ink: u64,
}

//#[allow(dead_code)]
//#[derive(Clone, Debug, Eq, PartialEq, SimpleSnakeNames)]
#[derive(Debug)]
pub enum HostioKind {
    UserEntrypoint {
        args_len: u32,
    },
    UserReturned {
        status: u32,
    },
    ReadArgs {
        args: Box<[u8]>,
    },
    WriteResult {
        result: Box<[u8]>,
    },
    ExitEarly {
        status: u32,
    },
    StorageLoadBytes32 {
        key: B256,
        value: B256,
    },
    StorageCacheBytes32 {
        key: B256,
        value: B256,
    },
    StorageFlushCache {
        clear: u8,
    },
    TransientLoadBytes32 {
        key: B256,
        value: B256,
    },
    TransientStoreBytes32 {
        key: B256,
        value: B256,
    },
    AccountBalance {
        address: Address,
        balance: U256,
    },
    AccountCode {
        address: Address,
        offset: u32,
        size: u32,
        code: Box<[u8]>,
    },
    AccountCodeSize {
        address: Address,
        size: u32,
    },
    AccountCodehash {
        address: Address,
        codehash: B256,
    },
    BlockBasefee {
        basefee: U256,
    },
    BlockCoinbase {
        coinbase: Address,
    },
    BlockGasLimit {
        limit: u64,
    },
    BlockNumber {
        number: u64,
    },
    BlockTimestamp {
        timestamp: u64,
    },
    Chainid {
        chainid: u64,
    },
    ContractAddress {
        address: Address,
    },
    EvmGasLeft {
        gas_left: u64,
    },
    EvmInkLeft {
        ink_left: u64,
    },
    PayForMemoryGrow {
        pages: u16,
    },
    MathDiv {
        a: U256,
        b: U256,
        result: U256,
    },
    MathMod {
        a: U256,
        b: U256,
        result: U256,
    },
    MathPow {
        a: U256,
        b: U256,
        result: U256,
    },
    MathAddMod {
        a: U256,
        b: U256,
        c: U256,
        result: U256,
    },
    MathMulMod {
        a: U256,
        b: U256,
        c: U256,
        result: U256,
    },
    MsgReentrant {
        reentrant: bool,
    },
    MsgSender {
        sender: Address,
    },
    MsgValue {
        value: B256,
    },
    NativeKeccak256 {
        preimage: Box<[u8]>,
        digest: B256,
    },
    TxGasPrice {
        gas_price: U256,
    },
    TxInkPrice {
        ink_price: u32,
    },
    TxOrigin {
        origin: Address,
    },
    ConsoleLog {
        text: String,
    },
    ConsoleLogText {
        text: Box<[u8]>,
    },
    CallContract {
        address: Address,
        data: Box<[u8]>,
        gas: u64,
        value: U256,
        outs_len: u32,
        status: u8,
        frame: TraceFrame,
    },
    DelegateCallContract {
        address: Address,
        data: Box<[u8]>,
        gas: u64,
        outs_len: u32,
        status: u8,
        frame: TraceFrame,
    },
    StaticCallContract {
        address: Address,
        data: Box<[u8]>,
        gas: u64,
        outs_len: u32,
        status: u8,
        frame: TraceFrame,
    },
    Create1 {
        code: Box<[u8]>,
        endowment: U256,
        address: Address,
        revert_data_len: u32,
    },
    Create2 {
        code: Box<[u8]>,
        endowment: U256,
        salt: B256,
        address: Address,
        revert_data_len: u32,
    },
    EmitLog {
        data: Box<[u8]>,
        topics: u32,
    },
    ReadReturnData {
        offset: u32,
        size: u32,
        data: Box<[u8]>,
    },
    ReturnDataSize {
        size: u32,
    },
    EVMCall {
        name: String,
        frame: TraceFrame,
    },
}
