#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

use stylus_sdk::{
    abi::Bytes, alloy_primitives::Address, prelude::*, stylus_core::calls::context::Call,
};

#[storage]
#[entrypoint]
pub struct SingleCall;

#[public]
impl SingleCall {
    pub fn execute(&self, target: Address, data: Bytes) -> Bytes {
        let result = self.vm().call(&Call::default(), target, &data);

        result.unwrap().into()
    }
}
