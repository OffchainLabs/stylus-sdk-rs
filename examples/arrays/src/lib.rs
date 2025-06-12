// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;

/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{
    alloy_primitives::{Address, U256},
    prelude::*,
};

// Define some persistent storage using the Solidity ABI.
// `Arrays` will be the entrypoint.
sol_storage! {
    #[entrypoint]
    pub struct Arrays {
        uint256[] arr;
        uint256[10] arr2; // fixed length array
        Info[] arr3; // struct array
    }

    pub struct Info {
        address setter;
        uint256 value;
    }
}

/// Declare that `Arrays` is a contract with the following external methods.
#[public]
impl Arrays {
    // dynamic array
    // push an element to the dynamic array
    pub fn push(&mut self, i: U256) {
        self.arr.push(i);
    }

    // get the element at the index
    pub fn get_element(&self, index: U256) -> U256 {
        self.arr.get(index).unwrap()
    }

    // get the length of the array
    pub fn get_arr_length(&self) -> U256 {
        U256::from(self.arr.len())
    }

    // remove will not change the length of the array
    pub fn remove(&mut self, index: U256) {
        let mut last_element = self.arr.setter(index).unwrap();
        last_element.erase()
    }

    // fixed length array
    // get an element from the fixed length array
    pub fn get_arr2_element(&self, index: U256) -> U256 {
        self.arr2.get(index).unwrap()
    }

    // get the fixed length array size
    pub fn get_arr2_length(&self) -> U256 {
        U256::from(self.arr2.len())
    }

    // set an element in the fixed length array
    pub fn set_arr2_value(&mut self, index: U256, value: U256) {
        self.arr2.setter(index).unwrap().set(value);
    }

    // struct array
    // push an element to the struct array
    pub fn push_arr3_info(&mut self, value: U256) {
        let msg_sender = self.vm().msg_sender();
        let mut new_info = self.arr3.grow();
        new_info.setter.set(msg_sender);
        new_info.value.set(value);
    }

    // get the length of the struct array
    pub fn get_arr3_length(&self) -> U256 {
        U256::from(self.arr3.len())
    }

    // get the value of the struct array at the index
    pub fn get_arr3_info(&self, index: U256) -> (Address, U256) {
        let info = self.arr3.get(index).unwrap();
        (info.setter.get(), info.value.get())
    }

    // Find the first index of the expected value in the array
    pub fn find_arr3_first_expected_value(&self, expected_value: U256) -> U256 {
        for i in 0..self.arr3.len() {
            let (_, value) = self.get_arr3_info(U256::from(i));
            if value == expected_value {
                return U256::from(i);
            }
        }
        // if not found, return the size of arr
        U256::from(self.arr3.len())
    }
}
