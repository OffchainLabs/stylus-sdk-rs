// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::{
    primitives::{Address, Bytes, U256},
    providers::Provider,
    rpc::types::state::AccountOverride,
};

use crate::{error::Result, precompiles};

/// Checks contract activation, returning the data fee.
// TODO: should this be in activate.rs?
pub async fn check_activate(code: Bytes, provider: impl Provider) -> Result<U256> {
    let arbwasm = precompiles::arb_wasm(provider);
    let random_sender_addr = Address::random();
    let spoofed_sender_account = AccountOverride::default().with_balance(U256::MAX);
    let spoofed_code = AccountOverride::default().with_code(code.clone());
    todo!()
}
