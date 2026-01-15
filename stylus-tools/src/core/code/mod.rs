// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Tagged WASM code, already processed and ready to be uploaded on-chain

use std::path::Path;

use alloy::primitives::B256;

use crate::core::code::wasm::{compress_wasm, process_wasm_file, CompressedWasm, WasmError};

pub mod contract;
pub mod fragments;
pub mod wasm;

/// Prefixes for code chunks on-chain
pub mod prefixes {
    /// EOF prefix used in Stylus compressed WASMs on-chain
    pub const EOF_NO_DICT: &[u8] = &[0xEF, 0xF0, 0x00, 0x00];
    /// Prefix used to denote a Stylus contract fragment
    pub const FRAGMENT: &[u8] = &[0xEF, 0xF0, 0x01];
    /// Prefix used to denote a Stylus contract root
    pub const ROOT_NO_DICT: &[u8] = &[0xEF, 0xF0, 0x02, 0x00];
}

/// WASM code that has been tagged with a prefix for identification by the nitro backend
#[derive(Debug)]
pub enum Code {
    /// Small contracts will fit within one code chunk
    Contract(contract::ContractCode),
    /// Larger contracts will be split into fragments
    Fragments(fragments::CodeFragments),
}

impl Code {
    pub fn from_wasm_file(
        filename: impl AsRef<Path>,
        project_hash: [u8; 32],
        max_code_size: u64,
    ) -> Result<Self, WasmError> {
        let processed = process_wasm_file(filename, project_hash)?;
        let compressed = compress_wasm(&processed)?;
        Ok(Self::split_if_large(&compressed, max_code_size))
    }

    /// Create code chunks, splitting the contract if it too large
    pub fn split_if_large(wasm: &CompressedWasm, max_code_size: u64) -> Self {
        if wasm.len() + prefixes::EOF_NO_DICT.len() <= max_code_size as usize {
            // Code will fit within one chunk
            Self::Contract(contract::ContractCode::new(wasm))
        } else {
            // Split code into appropriately sized fragments
            Self::Fragments(fragments::CodeFragments::new(wasm, max_code_size))
        }
    }

    /// Get codehash of contract or fragments
    pub fn codehash(&self) -> B256 {
        match self {
            Self::Contract(contract) => contract.codehash(),
            Self::Fragments(fragments) => fragments.codehash(),
        }
    }
}
