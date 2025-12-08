// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Tools for processing Wasm bytecode for Stylus contracts.

use std::{fs, io, path::Path};

use alloy::primitives::B256;
use wasmparser::BinaryReaderError;

use crate::utils::wasm;

/// Maximum brotli compression level used for Stylus contracts.
pub const BROTLI_COMPRESSION_LEVEL: u32 = 11;

/// EOF prefix used in Stylus compressed WASMs on-chain
pub const EOF_PREFIX_NO_DICT: &str = "EFF00000";

/// Name of the custom wasm section that is added to contracts deployed with cargo stylus
/// to include a hash of the Rust project's source files for reproducible verification of builds.
pub const PROJECT_HASH_SECTION_NAME: &str = "project_hash";

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
    let code = wasm::add_prefix(code, EOF_PREFIX_NO_DICT);

    Ok(ProcessedWasm { wasm, code })
}

#[derive(Debug)]
pub struct ProcessedWasm {
    pub wasm: Vec<u8>,
    pub code: Vec<u8>,
}

impl ProcessedWasm {
    pub fn codehash(&self) -> B256 {
        alloy::primitives::keccak256(&self.code)
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
