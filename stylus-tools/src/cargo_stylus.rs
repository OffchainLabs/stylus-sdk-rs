// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::primitives::Address;
use eyre::{bail, eyre, Result, WrapErr};
use regex::Regex;
use std::{ffi::OsStr, process::Command};

/// Deploy the contract in the current directory
pub fn deploy(rpc: &str, key: &str) -> Result<Address> {
    call_deploy(["-e", rpc, "--private-key", key])
}

/// Deploy the contract in the current directory passing the arguments to the constructor.
/// This function will fail if the contract doesn't have a constructor.
pub fn deploy_with_constructor(
    rpc: &str,
    key: &str,
    value: &str,
    args: &[&str],
) -> Result<Address> {
    let mut deploy_args = vec!["-e", rpc, "--private-key", key];
    if !value.is_empty() {
        deploy_args.push("--constructor-value");
        deploy_args.push(value);
    }
    deploy_args.push("--constructor-args");
    deploy_args.extend_from_slice(args);
    cfg_if::cfg_if! {
        // When running with integration tests, use the stylus deployer defined in the devnet
        // module. Otherwise, leave this field blank and use cargo-stylus' default address.
        if #[cfg(feature = "integration-tests")] {
            use crate::devnet::addresses::STYLUS_DEPLOYER;
            deploy_args.push("--deployer-address");
            let address = STYLUS_DEPLOYER.to_string();
            deploy_args.push(&address);
        }
    }
    call_deploy(&deploy_args)
}

fn call_deploy<I: IntoIterator<Item = S>, S: AsRef<OsStr>>(args: I) -> Result<Address> {
    let output = Command::new("cargo")
        .arg("stylus")
        .arg("deploy")
        .arg("--no-verify")
        .args(args)
        .output()
        .wrap_err("failed to run cargo deploy")?;
    if !output.status.success() {
        let err = String::from_utf8(output.stderr).unwrap_or("failed to decode error".to_owned());
        bail!("failed to run node: {}", err);
    }
    let out = String::from_utf8(output.stdout).wrap_err("failed to decode stdout")?;
    let out = strip_color(&out);
    let address = extract_deployed_address(&out)?;
    let address: Address = address
        .parse()
        .wrap_err("failed to parse deployment address")?;
    Ok(address)
}

fn strip_color(s: &str) -> String {
    let re = Regex::new(r"\x1b\[[0-9;]*[ABCDHJKSTfGmsu]").unwrap();
    re.replace_all(s, "").into_owned()
}

fn extract_deployed_address(s: &str) -> Result<&str> {
    for line in s.lines() {
        if let Some(rest) = line.strip_prefix("deployed code at address: ") {
            return Ok(rest);
        }
    }
    Err(eyre!("deployment address not found"))
}
