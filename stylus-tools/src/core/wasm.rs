// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Tools for processing Wasm bytecode for Stylus contracts.

use std::{fs, io, path::Path};

use alloy::primitives::{Address, B256};
use wasmparser::BinaryReaderError;

use crate::utils::wasm;

/// Maximum brotli compression level used for Stylus contracts.
pub const BROTLI_COMPRESSION_LEVEL: u32 = 11;

/// EOF prefix used in Stylus compressed WASMs on-chain
pub const EOF_PREFIX_NO_DICT: &str = "EFF00000";
/// Prefix used to denote a Stylus contract fragment
pub const FRAGMENT_PREFIX: &str = "EFF00100";
/// Prefix used to denote a Stylus contract root
pub const ROOT_PREFIX: &str = "EFF00200";

/// Name of the custom wasm section that is added to contracts deployed with cargo stylus
/// to include a hash of the Rust project's source files for reproducible verification of builds.
pub const PROJECT_HASH_SECTION_NAME: &str = "project_hash";

/// Maximum size for splitting a contract into fragments.
pub const MAX_FRAGMENT_SIZE: usize = 24_000;

// TODO: process_wasm() from memory
pub fn process_wasm_file(
    filename: impl AsRef<Path>,
    project_hash: [u8; 32],
) -> Result<ProcessedWasm, ProcessWasmFileError> {
    let wasm = fs::read(filename).map_err(ProcessWasmFileError::Read)?;
    let wasm = wasm::remove_dangling_references(wasm)?;
    let wasm = wasm::strip_user_metadata(wasm).map_err(ProcessWasmFileError::StripUserMetadata)?;
    let wasm = wasm::add_custom_section(wasm, PROJECT_HASH_SECTION_NAME, project_hash)
        .map_err(ProcessWasmFileError::AddProjectHash)?;
    let wasm = wasmer::wat2wasm(&wasm)
        .map_err(ProcessWasmFileError::Wat2Wasm)?
        .to_vec();

    let code = wasm::brotli_compress(wasm.as_slice(), BROTLI_COMPRESSION_LEVEL)
        .map_err(ProcessWasmFileError::BrotliCompress)?;
    let code = ProcessedWasmCode::split_if_large(code);
    Ok(ProcessedWasm { wasm, code })
}

#[derive(Debug)]
pub struct ProcessedWasm {
    pub wasm: Vec<u8>,
    pub code: ProcessedWasmCode,
}

impl ProcessedWasm {
    pub fn codehash(&self) -> B256 {
        // TODO: proper codehash?
        match &self.code {
            ProcessedWasmCode::Code(code) => alloy::primitives::keccak256(code),
            ProcessedWasmCode::Fragments(fragments) => alloy::primitives::keccak256(
                fragments.iter().flatten().cloned().collect::<Vec<_>>(),
            ),
        }
    }
}

/// WASM code which has been processed, compressed, and is ready to be deployed
#[derive(Clone, Debug)]
pub enum ProcessedWasmCode {
    /// Compressed code which fits within a single chunk does not need to be fragmented
    Code(Vec<u8>),
    /// Large WASM code will be split into fragments
    Fragments(Vec<Vec<u8>>),
}

impl ProcessedWasmCode {
    /// Split contract code if the compressed size is too large
    pub fn split_if_large(code: Vec<u8>) -> Self {
        if code.len() <= MAX_FRAGMENT_SIZE {
            Self::Code(wasm::add_prefix(code, EOF_PREFIX_NO_DICT))
        } else {
            Self::Fragments(
                code.chunks(MAX_FRAGMENT_SIZE)
                    .map(|c| wasm::add_prefix(c.to_vec(), FRAGMENT_PREFIX))
                    .collect(),
            )
        }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        match self {
            Self::Code(code) => code.len(),
            Self::Fragments(fragments) => fragments.iter().map(Vec::len).sum(),
        }
    }
}

/// Root contract which points to one or more fragments
#[derive(Debug)]
pub struct ContractRoot {
    contents: Vec<u8>,
}

impl ContractRoot {
    /// Create the root contract from uncompressed contract size and address list
    pub fn new(
        uncompressesd_contract_size: usize,
        addresses: impl IntoIterator<Item = Address>,
    ) -> Self {
        let mut code = uncompressesd_contract_size
            .to_be_bytes()
            .into_iter()
            .collect::<Vec<_>>();
        for address in addresses.into_iter() {
            code.extend(address);
        }
        Self {
            contents: wasm::add_prefix(code, ROOT_PREFIX),
        }
    }

    pub fn contents(&self) -> &[u8] {
        &self.contents
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProcessWasmFileError {
    #[error("error reading wasm file: {0}")]
    Read(io::Error),
    #[error("error removing dangling references: {0}")]
    RemoveDanglingReferences(#[from] wasm::RemoveDanglingReferencesError),
    #[error("error adding project hash to wasm file as a custom section: {0}")]
    AddProjectHash(BinaryReaderError),
    #[error("error stripping user metadata: {0}")]
    StripUserMetadata(BinaryReaderError),
    #[error("error converting Wat to Wasm: {0}")]
    Wat2Wasm(wat::Error),
    #[error("failed to compress Wasm bytes")]
    BrotliCompress(io::Error),
}
