// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::io;

use crate::core::build::BuildConfig;

pub fn write_initcode(_build_config: &BuildConfig, _output: impl io::Write) -> eyre::Result<()> {
    /*
        let (wasm, project_hash) = project::build_wasm_from_features(
            cfg.features.clone(),
            cfg.source_files_for_project_hash.clone(),
        )?;

        let (_, code) =
            project::compress_wasm(&wasm, project_hash).wrap_err("failed to compress WASM")?;

        let initcode = DeploymentCalldata::new(code);
        output.write(hex::encode(initcode.0).as_bytes())?;
    */
    Ok(())
}
