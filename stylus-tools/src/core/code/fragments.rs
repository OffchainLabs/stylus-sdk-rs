// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Support for fragmented Stylus contracts

use alloy::primitives::B256;

use crate::core::code::{prefixes, wasm::CompressedWasm};

/// Fragmented code, with the appropriate prefix bytes
#[derive(Debug)]
pub struct CodeFragment(pub Vec<u8>);

impl CodeFragment {
    /// Create code fragment from wasm "chunk", adding the appropriate prefix bytes
    pub fn new(wasm: &[u8]) -> Self {
        let mut code = Vec::with_capacity(prefixes::FRAGMENT.len() + wasm.len());
        code.extend(prefixes::FRAGMENT);
        code.extend(wasm);
        Self(code)
    }
}

/// Complete contract worth of code fragments, each prefixed with the appropriate bytes
#[derive(Debug)]
pub struct CodeFragments(pub Vec<CodeFragment>);

impl CodeFragments {
    /// Split wasm code into chunks according to given max size
    pub fn new(wasm: &CompressedWasm, max_code_size: u64) -> Self {
        let fragments = wasm
            .bytes()
            // Split into chunks, leaving room for the prefix as well
            .chunks(max_code_size as usize - prefixes::FRAGMENT.len())
            .map(CodeFragment::new)
            .collect();
        Self(fragments)
    }

    /// Get a slice containing all fragments
    pub fn as_slice(&self) -> &[CodeFragment] {
        &self.0
    }

    /// Codehash is hash of all fragments together
    pub fn codehash(&self) -> B256 {
        alloy::primitives::keccak256(
            self.0
                .iter()
                .map(|f| f.0.iter().cloned())
                .flatten()
                .collect::<Vec<_>>(),
        )
    }

    /// Length of all code chunks together
    pub fn codesize(&self) -> usize {
        self.0.iter().map(|f| f.0.len()).sum()
    }

    /// Number of fragments
    pub fn fragment_count(&self) -> usize {
        self.0.len()
    }
}
