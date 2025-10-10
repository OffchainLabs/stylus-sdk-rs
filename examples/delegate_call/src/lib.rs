// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(feature = "contract-client-gen", allow(unused_imports))]

extern crate alloc;

use alloy_sol_types::sol;
use stylus_sdk::{
    abi::Bytes,
    alloy_primitives::Address,
    call::{delegate_call, RawCall},
    prelude::*,
};

#[storage]
#[entrypoint]
pub struct ExampleContract;
// Declare events and Solidity error types
sol! {
    error DelegateCallFailed();
}

#[derive(SolidityError)]
pub enum DelegateCallErrors {
    DelegateCallFailed(DelegateCallFailed),
}

#[public]
impl ExampleContract {
    pub fn low_level_delegate_call(
        &mut self,
        calldata: Bytes,
        target: Address,
    ) -> Result<Vec<u8>, DelegateCallErrors> {
        unsafe {
            let config = Call::new_mutating(self);
            let result = delegate_call(self.vm(), config, target, &calldata)
                .map_err(|_| DelegateCallErrors::DelegateCallFailed(DelegateCallFailed {}))?;

            Ok(result)
        }
    }

    pub fn raw_delegate_call(
        &mut self,
        calldata: Vec<u8>,
        target: Address,
    ) -> Result<Vec<u8>, Vec<u8>> {
        let data = unsafe {
            RawCall::new_delegate(self.vm()) // configure a delegate call
                .gas(2100) // supply 2100 gas
                .limit_return_data(0, 32) // only read the first 32 bytes back
                .call(target, &calldata)?
        };

        Ok(data)
    }
}
