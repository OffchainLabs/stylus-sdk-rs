// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::primitives::Address;
use eyre::{bail, eyre, Result, WrapErr};
use regex::Regex;
use std::process::Command;

/// Deploy the contract in the current directory
pub fn deploy(rpc: &str, key: &str) -> Result<Address> {
    let output = Command::new("cargo")
        .arg("stylus")
        .arg("deploy")
        .arg("--no-verify")
        .arg("-e")
        .arg(rpc)
        .arg("--private-key")
        .arg(key)
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
