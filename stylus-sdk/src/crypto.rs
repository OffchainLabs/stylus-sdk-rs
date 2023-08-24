// Copyright 2022-2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use alloy_primitives::B256;

/// Efficiently computes the [`keccak256`] hash of the given preimage.
///
/// [`keccak256`]: https://en.wikipedia.org/wiki/SHA-3
pub fn keccak<T: AsRef<[u8]>>(bytes: T) -> B256 {
    alloy_primitives::keccak256(bytes)
}
