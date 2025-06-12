// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;

use alloy_sol_types::sol;
use stylus_sdk::{
    abi::Bytes,
    alloy_primitives::{Address, U256},
    call::RawCall,
    prelude::*,
};

#[storage]
#[entrypoint]
pub struct MultiCall;

// Declare events and Solidity error types
sol! {
    error ArraySizeNotMatch();
    error CallFailed(uint256 call_index);
}

#[derive(SolidityError)]
pub enum MultiCallErrors {
    ArraySizeNotMatch(ArraySizeNotMatch),
    CallFailed(CallFailed),
}

#[public]
impl MultiCall {
    pub fn multicall(
        &self,
        addresses: Vec<Address>,
        data: Vec<Bytes>,
    ) -> Result<Vec<Bytes>, MultiCallErrors> {
        let addr_len = addresses.len();
        let data_len = data.len();
        let mut results: Vec<Bytes> = Vec::new();
        if addr_len != data_len {
            return Err(MultiCallErrors::ArraySizeNotMatch(ArraySizeNotMatch {}));
        }
        for i in 0..addr_len {
            let result: Result<Vec<u8>, Vec<u8>> =
                unsafe { RawCall::new(self.vm()).call(addresses[i], data[i].to_vec().as_slice()) };
            let data = match result {
                Ok(data) => data,
                Err(_data) => {
                    return Err(MultiCallErrors::CallFailed(CallFailed {
                        call_index: U256::from(i),
                    }))
                }
            };
            results.push(data.into())
        }
        Ok(results)
    }
}
