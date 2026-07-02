// Copyright 2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! An intentionally large contract used to exercise fragmented deployment and verification.
//!
//! The contract embeds a sizable, incompressible blob so that its *compressed* wasm exceeds the
//! per-fragment code-size limit (24 KiB). This forces `cargo stylus deploy` down the fragmented
//! deployment path (multiple fragment contracts plus a root contract), which in turn lets the
//! integration test exercise `cargo stylus verify` against a real multi-fragment deployment.

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(feature = "contract-client-gen", allow(unused_imports))]

extern crate alloc;

use stylus_sdk::{alloy_primitives::B256, crypto, prelude::*};

/// Size of the embedded incompressible blob, in bytes.
///
/// At ~48 KiB the blob comfortably exceeds the 24 KiB per-fragment limit, so the compressed
/// contract is split into multiple fragments at deploy time.
const BLOB_LEN: usize = 48 * 1024;

/// Fills a byte array with high-entropy pseudo-random data using splitmix64.
///
/// splitmix64 has strong avalanche behavior, so its output is effectively incompressible — the
/// resulting data segment does not shrink under brotli and reliably pushes the contract past the
/// fragmentation threshold.
const fn build_blob() -> [u8; BLOB_LEN] {
    let mut blob = [0u8; BLOB_LEN];
    let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut i = 0;
    while i < BLOB_LEN {
        state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        let bytes = z.to_le_bytes();
        let mut j = 0;
        while j < 8 && i < BLOB_LEN {
            blob[i] = bytes[j];
            i += 1;
            j += 1;
        }
    }
    blob
}

/// The embedded incompressible blob, materialized into the deployed wasm's data segment.
static BLOB: [u8; BLOB_LEN] = build_blob();

#[storage]
#[entrypoint]
pub struct LargeContract;

#[public]
impl LargeContract {
    /// Returns the keccak256 hash of the embedded blob.
    ///
    /// keccak is a host call over every byte of `BLOB`, so the optimizer cannot fold it away or
    /// drop the data segment — guaranteeing the large, incompressible blob stays in the deployed
    /// wasm and the contract fragments.
    fn blob_hash(&self) -> B256 {
        crypto::keccak(&BLOB[..])
    }

    /// Returns the length of the embedded blob.
    fn blob_len(&self) -> u64 {
        BLOB.len() as u64
    }
}
