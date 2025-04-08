// Copyright 2025-2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! This module defines the internal state of the Stylus test VM.

use alloy_primitives::{Address, B256, U256};
use alloy_provider::{network::Ethereum, RootProvider};
use std::{collections::HashMap, sync::Arc};

use crate::constants::{DEFAULT_CHAIN_ID, DEFAULT_CONTRACT_ADDRESS, DEFAULT_SENDER};

/// Type aliases for the return values of mocked calls and deployments.
type CallReturn = Result<Vec<u8>, Vec<u8>>;
type DeploymentReturn = Result<Address, Vec<u8>>;
type MockCallWithAddress = (Address, Vec<u8>);
type DeploymentWithSalt = (Vec<u8>, Option<B256>);

/// Type alias for the RPC provider used in the test VM.
type RPCProvider = Arc<RootProvider<Ethereum>>;

/// Defines the internal state of the Stylus test VM for unit testing.
/// Internally, it tracks information such as mocked calls and their return values,
/// balances of addresses, and the storage of the contract being tested.
#[derive(Debug, Clone)]
pub struct VMState {
    pub storage: HashMap<U256, B256>,
    pub msg_sender: Address,
    pub contract_address: Address,
    pub chain_id: u64,
    pub reentrant: bool,
    pub block_number: u64,
    pub block_timestamp: u64,
    pub tx_origin: Option<Address>, // Defaults to msg sender if None.
    pub balances: HashMap<Address, U256>,
    pub code_storage: HashMap<Address, Vec<u8>>,
    pub gas_left: u64,
    pub ink_left: u64,
    pub msg_value: U256,
    pub block_gas_limit: u64,
    pub coinbase: Address,
    pub block_basefee: U256,
    pub tx_gas_price: U256,
    pub tx_ink_price: u32,
    pub call_returns: HashMap<MockCallWithAddress, CallReturn>,
    pub delegate_call_returns: HashMap<MockCallWithAddress, CallReturn>,
    pub static_call_returns: HashMap<MockCallWithAddress, CallReturn>,
    pub deploy_returns: HashMap<DeploymentWithSalt, DeploymentReturn>,
    pub emitted_logs: Vec<(Vec<B256>, Vec<u8>)>,
    pub provider: Option<RPCProvider>,
}

impl Default for VMState {
    fn default() -> Self {
        Self {
            storage: HashMap::new(),
            msg_sender: DEFAULT_SENDER,
            contract_address: DEFAULT_CONTRACT_ADDRESS,
            chain_id: DEFAULT_CHAIN_ID,
            reentrant: false,
            block_number: 0,
            block_timestamp: 0,
            balances: HashMap::new(),
            code_storage: HashMap::new(),
            gas_left: u64::MAX,
            ink_left: u64::MAX,
            msg_value: U256::ZERO,
            block_basefee: U256::from(1_000_000),
            block_gas_limit: 30_000_000,
            coinbase: DEFAULT_SENDER,
            tx_origin: None,
            tx_gas_price: U256::from(1),
            tx_ink_price: 1,
            call_returns: HashMap::new(),
            delegate_call_returns: HashMap::new(),
            static_call_returns: HashMap::new(),
            deploy_returns: HashMap::new(),
            emitted_logs: Vec::new(),
            provider: None,
        }
    }
}
