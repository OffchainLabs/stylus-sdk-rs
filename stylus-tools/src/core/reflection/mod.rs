// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Get information about a Stylus contract at build time
//!
//! This uses the mechanism of running a Stylus contract crate as a binary to return information
//! about the contract. This does not depend on a deployment of the contract.

use std::{path::PathBuf, process::Stdio};

pub use abi::abi;
pub use constructor::constructor;
use escargot::Cargo;

use crate::utils::sys;

mod abi;
mod constructor;

/// Feature that enables reflection when running the contract binary.
const FEATURE: &str = "export-abi";

fn reflect(command: &str, mut features: Vec<String>) -> Result<Vec<u8>, ReflectionError> {
    features.push(FEATURE.to_string());
    let output = Cargo::new()
        .into_command()
        .stderr(Stdio::inherit())
        .args(["run", "--quiet"])
        .args(["--target", &sys::host_arch()?])
        .args(["--features", &features.join(",")])
        .args(["--", command])
        .output()?;
    if !output.status.success() {
        let out = String::from_utf8_lossy(&output.stdout);
        let out = (!out.is_empty())
            .then_some(format!(" : {out}"))
            .unwrap_or_default();
        return Err(ReflectionError::FailedToRunContract(out));
    }
    Ok(output.stdout)
}

#[derive(Debug)]
pub struct ReflectionConfig {
    pub file: Option<PathBuf>,
    pub json: bool,
    pub rust_features: Option<Vec<String>>,
}

#[derive(Debug, thiserror::Error)]
pub enum ReflectionError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("{0}")]
    AlloyDynAbiParser(#[from] alloy::dyn_abi::parser::Error),
    #[error("{0}")]
    Host(#[from] rustc_host::Error),

    #[error("failed to run contract{0}")]
    FailedToRunContract(String),
}
