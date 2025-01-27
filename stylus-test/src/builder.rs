use std::{collections::HashMap, sync::Arc};

use alloy_primitives::{Address, B256, U256};
use ethers::providers::{Http, Provider};

use crate::{MockVMState, TestVM};

#[derive(Default)]
pub struct MockHostBuilder {
    sender: Option<Address>,
    value: Option<U256>,
    contract_address: Option<Address>,
    rpc_url: Option<String>,
    storage: Option<HashMap<U256, B256>>,
    provider: Option<Arc<Provider<Http>>>,
    block_num: Option<u64>,
}

impl MockHostBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn sender(mut self, sender: Address) -> Self {
        self.sender = Some(sender);
        self
    }
    pub fn value(mut self, value: U256) -> Self {
        self.value = Some(value);
        self
    }
    pub fn contract_address(mut self, address: Address) -> Self {
        self.contract_address = Some(address);
        self
    }
    pub fn rpc_url(mut self, url: String, block_num: Option<u64>) -> Self {
        self.rpc_url = Some(url);
        self.block_num = block_num;
        if let Some(url) = &self.rpc_url {
            if let Ok(provider) = Provider::<Http>::try_from(url.as_str()) {
                self.provider = Some(Arc::new(provider));
            }
        }
        self
    }
    pub fn build(self) -> Result<TestVM, &'static str> {
        let mut state = MockVMState::new();
        state.msg_sender = self.sender.unwrap_or(Address::ZERO);
        state.msg_value = self.value.unwrap_or_default();
        state.storage = self.storage.unwrap_or_default();
        state.block_number = self.block_num.unwrap_or_default();
        state.contract_address = self.contract_address.unwrap_or(Address::ZERO);
        state.provider = self.provider;
        Ok(TestVM::from(state))
    }
}
