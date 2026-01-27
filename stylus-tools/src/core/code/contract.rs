// Copyright 2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::primitives::{keccak256, Address, B256};

use crate::core::code::{prefixes, wasm::CompressedWasm};

/// Code for a contract which fits within MAX_CODE_SIZE
#[derive(Debug)]
pub struct ContractCode(pub Vec<u8>);

impl ContractCode {
    pub fn new(wasm: &CompressedWasm) -> Self {
        let mut code = Vec::with_capacity(prefixes::EOF_NO_DICT.len() + wasm.len());
        code.extend(prefixes::EOF_NO_DICT);
        code.extend(wasm.bytes());
        Self(code)
    }

    pub fn new_root_contract(
        uncompressed_wasm_size: u32,
        addresses: impl IntoIterator<Item = Address>,
    ) -> Self {
        let serialized_wasm_size = uncompressed_wasm_size.to_be_bytes();
        let addresses = addresses.into_iter();
        let mut code = Vec::with_capacity(
            prefixes::ROOT_NO_DICT.len()
                + serialized_wasm_size.len()
                + Address::len_bytes() * addresses.size_hint().1.unwrap_or(0),
        );
        code.extend(prefixes::ROOT_NO_DICT);
        code.extend(serialized_wasm_size);
        addresses.for_each(|a| code.extend(a));
        Self(code)
    }

    pub fn new_from_code(code: &[u8]) -> Self {
        Self(code.to_vec())
    }

    /// Get code bytes
    pub fn bytes(&self) -> &[u8] {
        &self.0
    }

    /// Codehash is keccak256 hash of the code bytes
    pub fn codehash(&self) -> B256 {
        keccak256(&self.0)
    }

    /// Length of the contract code
    pub fn codesize(&self) -> usize {
        self.0.len()
    }
}
