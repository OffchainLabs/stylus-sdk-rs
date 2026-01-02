// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Tools for processing Wasm bytecode for Stylus contracts.

use std::{fs, path::Path};

use crate::utils::wasm;

/// Maximum brotli compression level used for Stylus contracts.
pub const BROTLI_COMPRESSION_LEVEL: u32 = 11;

/// Name of the custom wasm section that is added to contracts deployed with cargo stylus
/// to include a hash of the Rust project's source files for reproducible verification of builds.
pub const PROJECT_HASH_SECTION_NAME: &str = "project_hash";

pub fn process_wasm_file(
    filename: impl AsRef<Path>,
    project_hash: [u8; 32],
) -> Result<ProcessedWasm, WasmError> {
    let wasm = fs::read(filename).map_err(WasmError::Read)?;
    process_wasm(&wasm, project_hash)
}

pub fn process_wasm(wasm: &[u8], project_hash: [u8; 32]) -> Result<ProcessedWasm, WasmError> {
    let wasm = wasm::remove_dangling_references(wasm)?;
    let wasm = wasm::strip_user_metadata(wasm).map_err(WasmError::StripUserMetadata)?;
    let wasm = wasm::add_custom_section(wasm, PROJECT_HASH_SECTION_NAME, project_hash)
        .map_err(WasmError::AddProjectHash)?;
    let wasm = wasmer::wat2wasm(&wasm)
        .map_err(WasmError::Wat2Wasm)?
        .to_vec();
    Ok(ProcessedWasm(wasm))
}

pub fn compress_wasm(wasm: &ProcessedWasm) -> Result<CompressedWasm, WasmError> {
    let wasm = wasm::brotli_compress(wasm.0.as_slice(), BROTLI_COMPRESSION_LEVEL)
        .map_err(WasmError::BrotliCompress)?;
    Ok(CompressedWasm(wasm))
}

#[derive(Debug)]
pub struct ProcessedWasm(Vec<u8>);

impl ProcessedWasm {
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Debug)]
pub struct CompressedWasm(Vec<u8>);

impl CompressedWasm {
    pub fn bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WasmError {
    #[error("error reading wasm file: {0}")]
    Read(std::io::Error),
    #[error("error removing dangling references: {0}")]
    RemoveDanglingReferences(#[from] wasm::RemoveDanglingReferencesError),
    #[error("error adding project hash to wasm file as a custom section: {0}")]
    AddProjectHash(wasmparser::BinaryReaderError),
    #[error("error stripping user metadata: {0}")]
    StripUserMetadata(wasmparser::BinaryReaderError),
    #[error("error converting Wat to Wasm: {0}")]
    Wat2Wasm(wat::Error),
    #[error("failed to compress Wasm bytes")]
    BrotliCompress(std::io::Error),
}
