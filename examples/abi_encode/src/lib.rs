// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;

use alloc::string::String;
/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{
    abi::Bytes,
    alloy_primitives::{Address, FixedBytes, U256},
    prelude::*,
};
// Because the naming of alloy_primitives and alloy_sol_types is the same, so we need to re-name the types in alloy_sol_types
use alloy_sol_types::{
    sol_data::{Address as SOLAddress, Bytes as SOLBytes, String as SOLString, *},
    SolType,
};
use sha3::{Digest, Keccak256};

// Define some persistent storage using the Solidity ABI.
// `Encoder` will be the entrypoint.
#[storage]
#[entrypoint]
pub struct Encoder;

impl Encoder {
    fn keccak256(&self, data: Bytes) -> FixedBytes<32> {
        // prepare hasher
        let mut hasher = Keccak256::new();
        // populate the data
        hasher.update(data);
        // hashing with keccack256
        let result = hasher.finalize();
        // convert the result hash to FixedBytes<32>
        let result_vec = result.to_vec();
        FixedBytes::<32>::from_slice(&result_vec)
    }
}

/// Declare that `Encoder` is a contract with the following external methods.
#[public]
impl Encoder {
    // Encode the data and hash it
    pub fn encode(
        &self,
        target: Address,
        value: U256,
        func: String,
        data: Bytes,
        timestamp: U256,
    ) -> Vec<u8> {
        // define sol types tuple
        type TxIdHashType = (SOLAddress, Uint<256>, SOLString, SOLBytes, Uint<256>);
        // set the tuple
        let tx_hash_data = (target, value, func, data, timestamp);
        // encode the tuple
        let tx_hash_data_encode = TxIdHashType::abi_encode_params(&tx_hash_data);
        tx_hash_data_encode
    }

    // Packed encode the data and hash it, the same result with the following one
    pub fn packed_encode(
        &self,
        target: Address,
        value: U256,
        func: String,
        data: Bytes,
        timestamp: U256,
    ) -> Vec<u8> {
        // define sol types tuple
        type TxIdHashType = (SOLAddress, Uint<256>, SOLString, SOLBytes, Uint<256>);
        // set the tuple
        let tx_hash_data = (target, value, func, data, timestamp);
        // encode the tuple
        let tx_hash_data_encode_packed = TxIdHashType::abi_encode_packed(&tx_hash_data);
        tx_hash_data_encode_packed
    }

    // Packed encode the data and hash it, the same result with the above one
    pub fn packed_encode_2(
        &self,
        target: Address,
        value: U256,
        func: String,
        data: Bytes,
        timestamp: U256,
    ) -> Vec<u8> {
        // set the data to array and concat it directly
        let tx_hash_data_encode_packed = [
            &target.to_vec(),
            &value.to_be_bytes_vec(),
            func.as_bytes(),
            &data.to_vec(),
            &timestamp.to_be_bytes_vec(),
        ]
        .concat();
        tx_hash_data_encode_packed
    }

    // The func example: "transfer(address,uint256)"
    pub fn encode_with_signature(&self, func: String, address: Address, amount: U256) -> Vec<u8> {
        type TransferType = (SOLAddress, Uint<256>);
        let tx_data = (address, amount);
        let data = TransferType::abi_encode_params(&tx_data);
        // Get function selector
        let hashed_function_selector = self.keccak256(func.as_bytes().to_vec().into());
        // Combine function selector and input data (use abi_packed way)
        let calldata = [&hashed_function_selector[..4], &data].concat();
        calldata
    }
}
