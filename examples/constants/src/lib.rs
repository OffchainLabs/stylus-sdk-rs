// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(feature = "contract-client-gen", allow(unused_imports))]

extern crate alloc;

use alloc::vec::Vec;

use stylus_sdk::alloy_primitives::Address;
use stylus_sdk::prelude::*;
use stylus_sdk::storage::StorageAddress;

#[cfg(not(feature = "contract-client-gen"))]
const OWNER: &str = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045";

#[storage]
#[entrypoint]
pub struct Contract {
    owner: StorageAddress,
}

#[public]
impl Contract {
    pub fn init(&mut self) -> Result<(), Vec<u8>> {
        // Parse the const &str as a local Address variable
        let owner_address = Address::parse_checksummed(OWNER, None).expect("Invalid address");

        // Save the result as the owner
        self.owner.set(owner_address);

        Ok(())
    }
    pub fn owner(&self) -> Result<Address, Vec<u8>> {
        let owner_address = self.owner.get();

        Ok(owner_address)
    }
}
