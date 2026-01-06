// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Utilities for working with Wasm bytecode.

use std::io::{self, Read};

use brotli2::read::BrotliEncoder;
use wasm_encoder::{Module, RawSection};
use wasmparser::{BinaryReaderError, Parser, Payload};

/// Add a section to the Wasm, if it does not already exist.
pub fn add_custom_section(
    wasm: impl Into<Vec<u8>>,
    name: &str,
    data: impl AsRef<[u8]>,
) -> Result<Vec<u8>, BinaryReaderError> {
    let mut wasm: Vec<u8> = wasm.into();
    if has_custom_section(&wasm, name)? {
        warn!(@grey, "Wasm file bytes already contains a custom section with {name}, not overwriting");
        return Ok(wasm);
    }
    wasm_gen::write_custom_section(&mut wasm, name, data.as_ref());
    Ok(wasm)
}

/// Check if a section exists in the Wasm.
pub fn has_custom_section(wasm: impl AsRef<[u8]>, name: &str) -> Result<bool, BinaryReaderError> {
    let parser = Parser::new(0);
    for payload in parser.parse_all(wasm.as_ref()) {
        if let Payload::CustomSection(reader) = payload? {
            if reader.name() == name {
                debug!(
                    @grey,
                    "Found the {name} custom section name {}",
                    hex::encode(reader.data())
                );
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Take Wasm bytecode and return its brotli compressed bytes.
pub fn brotli_compress(wasm: impl Read, compression_level: u32) -> io::Result<Vec<u8>> {
    let mut compressor = BrotliEncoder::new(wasm, compression_level);
    let mut compressed_bytes = vec![];
    compressor.read_to_end(&mut compressed_bytes)?;
    Ok(compressed_bytes)
}

/// Convert the WASM from binary to text and back to binary.
///
/// This trick removes any dangling mentions of reference types in the wasm body, which are not yet
/// supported by Arbitrum chain backends.
pub fn remove_dangling_references(
    wasm: impl AsRef<[u8]>,
) -> Result<Vec<u8>, RemoveDanglingReferencesError> {
    let wat_string = wasmprinter::print_bytes(wasm)?;
    let wasm = wasmer::wat2wasm(wat_string.as_bytes())?;
    Ok(wasm.to_vec())
}

#[derive(Debug, thiserror::Error)]
pub enum RemoveDanglingReferencesError {
    #[error("failed to convert Wasm to Wat: {0}")]
    Wasm2Wat(#[from] anyhow::Error),
    #[error("failed to convert Wat to Wasm: {0}")]
    Wat2Wasm(#[from] wat::Error),
}

/// Strip all custom and unknown sections from the Wasm binary.
///
/// This removes any user metadata which we do not want to leak as part of the final binary.
pub fn strip_user_metadata(
    wasm_file_bytes: impl AsRef<[u8]>,
) -> Result<Vec<u8>, BinaryReaderError> {
    let mut module = Module::new();
    // Parse the input WASM and iterate over the sections
    let parser = Parser::new(0);
    for payload in parser.parse_all(wasm_file_bytes.as_ref()) {
        match payload? {
            Payload::CustomSection { .. } => {
                // Skip custom sections to remove sensitive metadata
                // TODO: printing in utility function?
                debug!(@grey, "stripped custom section from user wasm to remove any sensitive data");
            }
            Payload::UnknownSection { .. } => {
                // Skip unknown sections that might not be sensitive
                // TODO: printing in utility function?
                debug!(@grey, "stripped unknown section from user wasm to remove any sensitive data");
            }
            item => {
                if let Some(section) = item.as_section() {
                    let (id, range) = section;
                    let data = &wasm_file_bytes.as_ref()[range];
                    let raw_section = RawSection { id, data };
                    module.section(&raw_section);
                }
            }
        }
    }
    Ok(module.finish())
}
