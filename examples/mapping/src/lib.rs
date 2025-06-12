// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;

use stylus_sdk::storage::*;
/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{
    alloy_primitives::{Address, U256},
    prelude::*,
};

#[storage]
#[entrypoint]
pub struct Mapping {
    my_map: StorageMap<Address, StorageBool>,
    my_nested_map: StorageMap<U256, StorageMap<Address, StorageBool>>,
}

// You can also define mapping storage using the Solidity way.
// sol_storage! {
//     #[entrypoint]
//     pub struct Mapping {
//         mapping(address => bool) my_map;
//         mapping(uint256 => mapping(address => bool)) my_nested_map;
//     }
// }

/// Declare that `Mapping` is a contract with the following external methods.
#[public]
impl Mapping {
    // First is the simple map ========================================

    pub fn get_my_map(&self, target: Address) -> bool {
        // Mapping always returns a value.
        // If the value was never set, it will return the default value.
        self.my_map.get(target)
    }

    pub fn set_my_map(&mut self, target: Address, new_value: bool) {
        // Update the value at this address
        self.my_map.setter(target).set(new_value);
    }

    pub fn remove_my_map(&mut self, target: Address) {
        // Reset the value to the default value.
        self.my_map.delete(target);
    }

    // Next is the nested map ========================================

    pub fn get_my_nested_map(&self, index: U256, target: Address) -> bool {
        // Mapping always returns a value.
        // If the value was never set, it will return the default value.
        self.my_nested_map.get(index).get(target)
    }

    pub fn set_my_nested_map(&mut self, index: U256, target: Address, new_value: bool) {
        // Update the value at this address
        self.my_nested_map
            .setter(index)
            .setter(target)
            .set(new_value);
    }

    pub fn remove_my_nested_map(&mut self, index: U256, target: Address) {
        // Reset the value to the default value.
        self.my_nested_map.setter(index).delete(target);
    }
}
