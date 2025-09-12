// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Export Stylus contract ABIs in Solidity format.

#![allow(dead_code)]

use std::{
    io::Write,
    process::{self, Command, Stdio},
};

use alloy::json_abi::Constructor;
use eyre::{Result, WrapErr};

use crate::{
    core::reflection::ReflectionConfig,
    utils::{solc, sys},
};

/// Exports Solidity ABIs by running the contract natively.
pub fn export_abi(package: &str, config: &ReflectionConfig) -> Result<()> {
    if config.json {
        solc::check_exists()?;
    }

    let features = config
        .rust_features
        .clone()
        .map(|feature_list| feature_list.join(","));
    let mut output = run_export(package, "abi", features)?;

    // convert the ABI to a JSON file via solc
    if config.json {
        let solc = Command::new("solc")
            .stdin(Stdio::piped())
            .stderr(Stdio::inherit())
            .stdout(Stdio::piped())
            .arg("--abi")
            .arg("-")
            .spawn()
            .wrap_err("failed to run solc")?;

        let mut stdin = solc.stdin.as_ref().unwrap();
        stdin.write_all(&output)?;
        output = solc.wait_with_output()?.stdout;
    }

    let mut out = sys::file_or_stdout(config.file.clone())?;
    out.write_all(&output)?;
    Ok(())
}

/// Print the constructor signature
pub fn print_constructor(package: &str, config: &ReflectionConfig) -> Result<()> {
    let features = config
        .rust_features
        .clone()
        .map(|feature_list| feature_list.join(","));
    let output = run_export(package, "constructor", features)?;
    if !std::str::from_utf8(&output)?.starts_with("constructor") {
        return Ok(());
    }
    let mut file = sys::file_or_stdout(config.file.clone())?;
    file.write_all(&output)?;
    Ok(())
}

/// Gets the constructor signature of the Stylus contract using the export binary.
/// If the contract doesn't have a constructor, returns None.
pub fn get_constructor_signature(package: &str) -> Result<Option<Constructor>> {
    greyln!("checking whether the contract has a constructor...");
    let output = run_export(package, "constructor", None)?;
    let output = String::from_utf8(output)?;
    parse_constructor(&output)
}

fn run_export(package: &str, command: &str, features: Option<String>) -> Result<Vec<u8>> {
    let target = format!("--target={}", sys::host_arch()?);
    let features = format!("--features=export-abi,{}", features.unwrap_or_default());

    let output = Command::new("cargo")
        .stderr(Stdio::inherit())
        .arg("run")
        .arg("--package")
        .arg(package)
        .arg("--quiet")
        .arg(features)
        .arg(target)
        .arg("--")
        .arg(command)
        .output()?;
    if !output.status.success() {
        let out = String::from_utf8_lossy(&output.stdout);
        let out = (!out.is_empty())
            .then_some(format!(": {out}"))
            .unwrap_or_default();
        egreyln!("failed to run contract {out}");
        process::exit(1);
    }
    Ok(output.stdout)
}

fn parse_constructor(signature: &str) -> Result<Option<Constructor>> {
    let signature = signature.trim();
    if !signature.starts_with("constructor") {
        // If the signature doesn't start with constructor, it is either an old SDK version that
        // doesn't support it or the contract doesn't have one. So, it is safe to return None.
        Ok(None)
    } else {
        Constructor::parse(signature)
            .map(Some)
            .map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::json_abi::Param;

    #[test]
    fn parse_constructors() {
        let test_cases = vec![
            (
                "constructor()",
                Some(Constructor {
                    inputs: vec![],
                    state_mutability: alloy::json_abi::StateMutability::NonPayable,
                }),
            ),
            (
                "constructor(uint256 foo)",
                Some(Constructor {
                    inputs: vec![Param {
                        ty: "uint256".to_owned(),
                        name: "foo".to_owned(),
                        components: vec![],
                        internal_type: None,
                    }],
                    state_mutability: alloy::json_abi::StateMutability::NonPayable,
                }),
            ),
            (
                "constructor((uint256, uint256) foo, uint8[] memory arr) payable",
                Some(Constructor {
                    inputs: vec![
                        Param {
                            ty: "tuple".to_owned(),
                            name: "foo".to_owned(),
                            components: vec![
                                Param {
                                    ty: "uint256".to_owned(),
                                    name: "".to_owned(),
                                    components: vec![],
                                    internal_type: None,
                                },
                                Param {
                                    ty: "uint256".to_owned(),
                                    name: "".to_owned(),
                                    components: vec![],
                                    internal_type: None,
                                },
                            ],
                            internal_type: None,
                        },
                        Param {
                            ty: "uint8[]".to_owned(),
                            name: "arr".to_owned(),
                            components: vec![],
                            internal_type: None,
                        },
                    ],
                    state_mutability: alloy::json_abi::StateMutability::Payable,
                }),
            ),
            ("", None),
            (
                "/**
 * This file was automatically generated by Stylus and represents a Rust program.
 * For more information, please see [The Stylus SDK](https://github.com/OffchainLabs/stylus-sdk-rs).
 */

// SPDX-License-Identifier: MIT-OR-APACHE-2.0
pragma solidity ^0.8.23;

interface ICounter  {
    function number() external view returns (uint256);

    function setNumber(uint256 new_number) external;
}",
                None,
            ),
        ];
        for (signature, expected) in test_cases {
            let constructor = parse_constructor(signature).expect("failed to parse");
            assert_eq!(constructor, expected);
        }
    }
}
