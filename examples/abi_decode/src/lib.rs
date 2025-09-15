// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;

/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{
    alloy_primitives::{Address, U256},
    prelude::*,
};
// Because the naming of `alloy_primitives` and `alloy_sol_types` is the same, we need to rename the types in `alloy_sol_types`.
use alloy_sol_types::{
    sol,
    sol_data::{Address as SOLAddress, *},
    SolType,
};

// Define error
sol! {
    error DecodedFailed();
}

// Error types for the MultiSig contract
#[derive(SolidityError)]
pub enum DecoderError {
    DecodedFailed(DecodedFailed),
}

#[storage]
#[entrypoint]
pub struct Decoder;

/// Declare that `Decoder` is a contract with the following external methods.
#[public]
impl Decoder {
    // This should always return true
    pub fn encode_and_decode(&self, address: Address, amount: U256) -> Result<bool, DecoderError> {
        // define sol types tuple
        type TxIdHashType = (SOLAddress, Uint<256>);
        // set the tuple
        let tx_hash_data = (address, amount);
        // encode the tuple
        let tx_hash_data_encode = TxIdHashType::abi_encode_params(&tx_hash_data);

        // Check the result
        match TxIdHashType::abi_decode_params(&tx_hash_data_encode) {
            Ok(res) => Ok(res == tx_hash_data),
            Err(_) => Err(DecoderError::DecodedFailed(DecodedFailed {})),
        }
    }
}
