// Copyright 2025-2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Defines a builder struct that can create a [`crate::TestVM`] instance
//! with convenient overrides for unit testing Stylus contracts.

use std::{collections::HashMap, sync::Arc};

use alloy_primitives::{Address, B256, U256};
use alloy_provider::{network::Ethereum, RootProvider};
use url::Url;

use crate::{state::VMState, TestVM};

/// Builder for constructing a [`crate::TestVM`] used for unit testing Stylus contracts built with the Stylus SDK.
/// Allows for convenient customization of the contract's address, sender address, message value, and RPC
/// URL if state forking is desired. These values and more can still be customized if the builder is not used,
/// by instead invoking the corresponding method on the TestVM struct such as `vm.set_value(value)`.
///
/// # Example
/// ```
/// use stylus_test::{TestVM, TestVMBuilder};
/// use alloy_primitives::{address, Address, U256};
///
/// let vm: TestVM = TestVMBuilder::new()
///     .sender(address!("dCE82b5f92C98F27F116F70491a487EFFDb6a2a9"))
///     .contract_address(address!("DeaDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF"))
///     .value(U256::from(1))
///     .rpc_url("http://localhost:8547")
///     .build();
/// ```
#[derive(Default)]
pub struct TestVMBuilder {
    sender: Option<Address>,
    value: Option<U256>,
    contract_address: Option<Address>,
    rpc_url: Option<String>,
    storage: Option<HashMap<U256, B256>>,
    provider: Option<Arc<RootProvider<Ethereum>>>,
    block_num: Option<u64>,
    chain_id: Option<u64>,
}

impl TestVMBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    /// Sets the sender address for contract invocations.
    pub fn sender(mut self, sender: Address) -> Self {
        self.sender = Some(sender);
        self
    }
    /// Sets the block number.
    pub fn block_number(mut self, block_num: u64) -> Self {
        self.block_num = Some(block_num);
        self
    }
    /// Sets the chain id.
    pub fn chain_id(mut self, id: u64) -> Self {
        self.chain_id = Some(id);
        self
    }
    /// Sets the storage mapping.
    pub fn storage(mut self, mapping: HashMap<U256, B256>) -> Self {
        self.storage = Some(mapping);
        self
    }
    /// Sets the msg value for contract invocations.
    pub fn value(mut self, value: U256) -> Self {
        self.value = Some(value);
        self
    }
    /// Sets the contract address.
    pub fn contract_address(mut self, address: Address) -> Self {
        self.contract_address = Some(address);
        self
    }
    /// Sets the RPC URL to a Stylus-enabled Arbitrum chain for storage forking.
    /// If specified, any calls to load storage will be made to the RPC URL at the TestVM's specified
    /// contract address.
    pub fn rpc_url(mut self, url: &str) -> Self {
        let url = match Url::parse(url) {
            Ok(url) => url,
            Err(e) => {
                panic!("Invalid RPC URL specified: {e}");
            }
        };
        self.rpc_url = Some(url.to_string());
        self.provider = Some(Arc::new(RootProvider::new_http(url)));
        self
    }
    /// Returns and TestVM instance from the builder with the specified parameters.
    pub fn build(self) -> TestVM {
        let mut state = VMState::default();
        if let Some(sender) = self.sender {
            state.msg_sender = sender;
        }
        if let Some(value) = self.value {
            state.msg_value = value;
        }
        if let Some(storage) = self.storage {
            state.storage = storage;
        }
        if let Some(block_number) = self.block_num {
            state.block_number = block_number;
        }
        if let Some(id) = self.chain_id {
            state.chain_id = id;
        }
        if let Some(contract_address) = self.contract_address {
            state.contract_address = contract_address;
        }
        state.provider = self.provider;
        TestVM::from(state)
    }
}
