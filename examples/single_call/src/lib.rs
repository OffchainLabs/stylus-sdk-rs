#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

use stylus_sdk::{abi::Bytes, alloy_primitives::Address, prelude::*};

#[storage]
#[entrypoint]
pub struct SingleCall;

#[public]
impl SingleCall {
    pub fn execute(&self, target: Address, data: Bytes) -> Bytes {
        unsafe {
            let result = RawCall::new(self.vm()).call(target, &data);
            result.unwrap().into()
        }
    }
}
