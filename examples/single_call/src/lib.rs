#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

use stylus_sdk::{abi::Bytes, alloy_primitives::Address, call::RawCall, prelude::*};

#[storage]
#[entrypoint]
pub struct SingleCall;

#[public]
impl SingleCall {
    pub fn execute(&self, target: Address, data: Bytes) -> Bytes {
        let result = RawCall::new().call(target, data.to_vec().as_slice());

        result.unwrap().into()
    }
}
