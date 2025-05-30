// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;

use stylus_sdk::{
    alloy_primitives::{Address, FixedBytes, I256, U256},
    prelude::*,
    storage::*,
};

#[storage]
#[entrypoint]
pub struct Data {
    my_bool: StorageBool,
    my_address: StorageAddress,
    my_uint: StorageU256,
    my_signed: StorageI256,
    my_fixed_bytes: StorageFixedBytes<4>,
    my_bytes: StorageBytes,
    my_string: StorageString,
    my_vec: StorageVec<StorageU256>,
}

#[public]
impl Data {
    // Getters
    pub fn get_bool(&self) -> bool {
        self.my_bool.get()
    }

    pub fn get_address(&self) -> Address {
        self.my_address.get()
    }

    pub fn get_uint(&self) -> U256 {
        self.my_uint.get()
    }

    pub fn get_signed(&self) -> I256 {
        self.my_signed.get()
    }

    pub fn get_fixed_bytes(&self) -> FixedBytes<4> {
        self.my_fixed_bytes.get()
    }

    pub fn get_bytes(&self) -> Vec<u8> {
        self.my_bytes.get_bytes()
    }

    pub fn get_byte_from_bytes(&self, index: U256) -> u8 {
        self.my_bytes.get(index).unwrap()
    }

    pub fn get_string(&self) -> String {
        self.my_string.get_string()
    }

    pub fn get_vec(&self, index: U256) -> U256 {
        self.my_vec.get(index).unwrap()
    }

    // Setters
    pub fn set_bool(&mut self, value: bool) {
        self.my_bool.set(value);
    }

    pub fn set_address(&mut self, value: Address) {
        self.my_address.set(value);
    }

    pub fn set_uint(&mut self, value: U256) {
        self.my_uint.set(value);
    }

    pub fn set_signed(&mut self, value: I256) {
        self.my_signed.set(value);
    }

    pub fn set_fixed_bytes(&mut self, value: FixedBytes<4>) {
        self.my_fixed_bytes.set(value);
    }

    pub fn set_bytes(&mut self, value: Vec<u8>) {
        self.my_bytes.set_bytes(value);
    }

    pub fn push_byte_to_bytes(&mut self, value: u8) {
        self.my_bytes.push(value);
    }

    pub fn set_string(&mut self, value: String) {
        self.my_string.set_str(value);
    }

    pub fn push_vec(&mut self, value: U256) {
        self.my_vec.push(value);
    }
}
