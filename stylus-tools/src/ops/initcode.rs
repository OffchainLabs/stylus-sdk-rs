// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::{env, io};

use alloy::primitives::Address;

use crate::core::{
    build::{build_contract, BuildConfig},
    chain::ChainConfig,
    code::{
        contract::ContractCode,
        wasm::{compress_wasm, process_wasm_file},
        Code,
    },
    deployment::prelude::DeploymentCalldata,
    project::{contract::Contract, hash_project, ProjectConfig},
};

pub fn write_initcode(
    contract: &Contract,
    build_config: &BuildConfig,
    chain_config: &ChainConfig,
    project_config: &ProjectConfig,
    mut output: impl io::Write,
) -> eyre::Result<()> {
    let wasm_file = build_contract(contract, build_config)?;
    let dir = env::current_dir()?;
    let project_hash = hash_project(dir, project_config, build_config)?;
    let processed = process_wasm_file(&wasm_file, project_hash)?;
    let compressed = compress_wasm(&processed)?;
    let code = Code::split_if_large(&compressed, chain_config.max_code_size);
    let contract = match code {
        Code::Contract(contract) => contract,
        Code::Fragments(fragments) => ContractCode::new_root_contract(
            processed.len(),
            fragments.as_slice().iter().map(|_| Address::ZERO),
        ),
    };
    let initcode = DeploymentCalldata::new(contract.bytes());
    output.write_all(hex::encode(initcode.0).as_bytes())?;
    Ok(())
}
