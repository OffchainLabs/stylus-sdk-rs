// Copyright 2022-2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

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

/// Represents a contract invocation outcome.
pub type ArbResult = Result<Vec<u8>, Vec<u8>>;
