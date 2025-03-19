// Copyright 2025-2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Defaults used by the [`crate::TestVM`] for unit testing Stylus contracts.

use alloy_primitives::{address, Address};

/// Default sender address used by the [`crate::TestVM`] for unit testing Stylus contracts.
pub const DEFAULT_SENDER: Address = address!("DeaDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF");

/// Default contract address used by the [`crate::TestVM`] for unit testing Stylus contracts.
pub const DEFAULT_CONTRACT_ADDRESS: Address = address!("dCE82b5f92C98F27F116F70491a487EFFDb6a2a9");

/// Default chain id used by the [`crate::TestVM`] for unit testing Stylus contracts.
pub const DEFAULT_CHAIN_ID: u64 = 42161; // Arbitrum One.

/// Default basefee used by the [`crate::TestVM`] for unit testing Stylus contracts.
pub const DEFAULT_BASEFEE: u64 = 1_000_000;

/// Default block gas limit used by the [`crate::TestVM`] for unit testing Stylus contracts.
pub const DEFAULT_BLOCK_GAS_LIMIT: u64 = 30_000_000;
