// Copyright 2022-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! The Stylus SDK.
//!
//! The Stylus SDK makes it easy to develop Solidity ABI-equivalent Stylus contracts in Rust.
//! Included is a full suite of types and shortcuts that abstract away the details of Solidity's storage layout,
//! method selectors, affordances, and more, making it easy to *just write Rust*.
//! For a guided exploration of the features, please see the comprehensive [Feature Overview][overview].
//!
//! Some of the features available in the SDK include:
//! - **Generic**, storage-backed Rust types for programming **Solidity-equivalent** smart contracts with optimal
//! storage caching.
//! - Simple macros for writing **language-agnostic** methods and entrypoints.
//! - Automatic export of Solidity interfaces for interoperability across programming languages.
//! - Powerful **primitive types** backed by the feature-rich [Alloy][alloy].
//!
//! Rust programs written with the Stylus SDK can **call and be called** by Solidity smart contracts
//! due to ABI equivalence with Ethereum programming languages. In fact, existing Solidity DEXs can list Rust
//! tokens without modification, and vice versa.
//!
//! [overview]: https://docs.arbitrum.io/stylus/reference/rust-sdk-guide
//! [alloy]: https://docs.rs/alloy-primitives/latest/alloy_primitives/

#![doc(html_favicon_url = "https://arbitrum.io/assets/stylus/Arbitrum_Stylus-Logomark.png")]
#![doc(html_logo_url = "https://arbitrum.io/assets/stylus/Arbitrum_Stylus-Logomark.png")]
#![warn(missing_docs)]
// Only allow the standard library in tests and for exports
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

extern crate alloc;

pub use alloy_primitives;
pub use alloy_sol_types;
pub use hex;
pub use keccak_const;
pub use stylus_proc;

#[macro_use]
pub mod abi;

#[macro_use]
pub mod debug;

pub mod block;
pub mod call;
pub mod contract;
pub mod crypto;
pub mod deploy;
pub mod evm;
pub mod msg;
pub mod prelude;
pub mod storage;
pub mod tx;
pub mod types;

mod util;

#[cfg(feature = "hostio")]
pub mod hostio;

#[cfg(not(feature = "hostio"))]
mod hostio;

use alloc::vec::Vec;

/// Represents a contract invocation outcome.
pub type ArbResult = Result<Vec<u8>, Vec<u8>>;
