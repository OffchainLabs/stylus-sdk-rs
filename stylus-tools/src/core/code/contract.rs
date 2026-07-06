// Copyright 2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::primitives::{keccak256, Address, B256};

use crate::core::code::{prefixes, wasm::CompressedWasm};

/// Error returned when [`ContractCode::parse_root_contract`] is given code that is not a
/// well-formed root contract.
#[derive(Debug, thiserror::Error)]
pub enum RootContractError {
    #[error("code is not a root contract (missing root prefix)")]
    NotRootContract,
    #[error("root contract code is too short to contain the uncompressed wasm size")]
    Truncated,
    #[error("root contract fragment-address section length is not a multiple of an address")]
    InvalidAddressSection,
}

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

    /// Parse a root contract's runtime code into its uncompressed wasm size and the addresses of
    /// its fragments. This is the inverse of [`ContractCode::new_root_contract`].
    ///
    /// Returns an error if `code` is not a well-formed root contract: wrong prefix, too short to
    /// hold the wasm size, or a fragment-address section whose length is not a multiple of an
    /// address.
    pub fn parse_root_contract(code: &[u8]) -> Result<(u32, Vec<Address>), RootContractError> {
        let rest = code
            .strip_prefix(prefixes::ROOT_NO_DICT)
            .ok_or(RootContractError::NotRootContract)?;
        if rest.len() < 4 {
            return Err(RootContractError::Truncated);
        }
        let (size_bytes, address_bytes) = rest.split_at(4);
        let uncompressed_wasm_size = u32::from_be_bytes(size_bytes.try_into().expect("4 bytes"));
        if address_bytes.len() % Address::len_bytes() != 0 {
            return Err(RootContractError::InvalidAddressSection);
        }
        let addresses = address_bytes
            .chunks_exact(Address::len_bytes())
            .map(Address::from_slice)
            .collect();
        Ok((uncompressed_wasm_size, addresses))
    }

    /// Get code bytes
    pub fn as_slice(&self) -> &[u8] {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_contract_round_trip() {
        let addresses = vec![
            Address::with_last_byte(1),
            Address::with_last_byte(2),
            Address::with_last_byte(3),
        ];
        let size = 123_456u32;
        let root = ContractCode::new_root_contract(size, addresses.clone());

        let (parsed_size, parsed_addresses) =
            ContractCode::parse_root_contract(root.as_slice()).unwrap();
        assert_eq!(parsed_size, size);
        assert_eq!(parsed_addresses, addresses);
    }

    #[test]
    fn root_contract_with_no_fragments_round_trips() {
        let root = ContractCode::new_root_contract(0, std::iter::empty());
        let (size, addresses) = ContractCode::parse_root_contract(root.as_slice()).unwrap();
        assert_eq!(size, 0);
        assert!(addresses.is_empty());
    }

    #[test]
    fn parse_rejects_wrong_prefix() {
        let mut code = prefixes::EOF_NO_DICT.to_vec();
        code.extend([0u8; 4]);
        assert!(matches!(
            ContractCode::parse_root_contract(&code),
            Err(RootContractError::NotRootContract)
        ));
    }

    #[test]
    fn parse_rejects_truncated_size() {
        let mut code = prefixes::ROOT_NO_DICT.to_vec();
        code.extend([0u8; 3]);
        assert!(matches!(
            ContractCode::parse_root_contract(&code),
            Err(RootContractError::Truncated)
        ));
    }

    #[test]
    fn parse_rejects_bad_address_section() {
        let mut code = prefixes::ROOT_NO_DICT.to_vec();
        code.extend([0u8; 4]); // wasm size
        code.extend([0u8; 19]); // not a multiple of an address (20 bytes)
        assert!(matches!(
            ContractCode::parse_root_contract(&code),
            Err(RootContractError::InvalidAddressSection)
        ));
    }
}
