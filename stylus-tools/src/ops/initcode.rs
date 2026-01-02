// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::{env, io};

use crate::core::{
    build::{build_contract, BuildConfig},
    code::Code,
    deployment::prelude::DeploymentCalldata,
    project::{contract::Contract, hash_project, ProjectConfig},
};

pub fn write_initcode(
    contract: &Contract,
    build_config: &BuildConfig,
    project_config: &ProjectConfig,
    mut output: impl io::Write,
) -> eyre::Result<()> {
    let wasm_file = build_contract(contract, build_config)?;
    let dir = env::current_dir()?;
    let project_hash = hash_project(dir, project_config, build_config)?;
    let code = Code::from_wasm_file(wasm_file, project_hash, build_config.max_code_size)?;
    let initcode = match &code {
        Code::Contract(contract) => DeploymentCalldata::new(contract.bytes()),
        Code::Fragments(_fragments) => todo!("support fragments for initcode"),
    };
    output.write_all(hex::encode(initcode.0).as_bytes())?;
    Ok(())
}
