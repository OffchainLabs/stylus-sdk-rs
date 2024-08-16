// Copyright 2022-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

//! VM-accelerated cryptography.
//!
//! See also [`block`](crate::block), [`contract`](crate::contract), [`evm`](crate::evm),
//! [`msg`](crate::msg), and [`tx`](crate::tx).
//!
//! ```no_run
//! use stylus_sdk::crypto;
//! use stylus_sdk::alloy_primitives::address;
//!
//! let preimage = address!("361594F5429D23ECE0A88E4fBE529E1c49D524d8");
//! let hash = crypto::keccak(&preimage);
//! ```

use alloy_primitives::B256;

/// Efficiently computes the [`keccak256`] hash of the given preimage.
///
/// [`keccak256`]: https://en.wikipedia.org/wiki/SHA-3
pub fn keccak<T: AsRef<[u8]>>(bytes: T) -> B256 {
    alloy_primitives::keccak256(bytes)
}
