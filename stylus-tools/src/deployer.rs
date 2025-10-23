// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::primitives::{Address, TxHash};
use eyre::{bail, eyre, Result, WrapErr};
use regex::Regex;
use std::borrow::ToOwned;
use std::iter::once;
use std::{env, path::Path, process::Command};
use typed_builder::TypedBuilder;

/// Defines the configuration for deploying a Stylus contract.
/// After setting the parameters, call `Deployer::deploy` to perform the deployement.
#[derive(TypedBuilder)]
#[builder(field_defaults(default, setter(into)))]
pub struct Deployer {
    #[builder(!default)]
    rpc: String,

    dir: Option<String>,

    #[cfg_attr(
        feature = "integration-tests",
        builder(default = crate::devnet::DEVNET_PRIVATE_KEY.to_owned())
    )]
    #[cfg_attr(not(feature = "integration-tests"), builder(!default))]
    private_key: String,

    #[cfg_attr(
        feature = "integration-tests",
        builder(default = Some(crate::devnet::addresses::STYLUS_DEPLOYER.to_string()))
    )]
    stylus_deployer: Option<String>,

    constructor_value: Option<String>,

    constructor_args: Option<Vec<String>>,
}

impl Deployer {
    pub fn estimate_gas(&self) -> Result<f64> {
        let deploy_args = self.deploy_args()?;
        let out = call(
            &self.dir,
            "deploy",
            once("--estimate-gas".to_owned()).chain(deploy_args),
        )?;

        extract_gas_estimate_result(&out)
    }

    // Deploy the Stylus contract returning the contract address.
    pub fn deploy(&self) -> Result<(Address, TxHash, f64)> {
        let deploy_args = self.deploy_args()?;

        let out = call(&self.dir, "deploy", deploy_args)?;
        let (address, tx_hash, gas_estimate) = extract_deploy_result(&out)?;
        let address: Address = address
            .parse()
            .wrap_err("failed to parse deployment address")?;
        let tx_hash: TxHash = tx_hash
            .parse()
            .wrap_err("failed to parse deployment tx hash")?;
        Ok((address, tx_hash, gas_estimate))
    }

    fn deploy_args(&self) -> Result<Vec<String>> {
        let mut deploy_args: Vec<String> = vec![
            "--no-verify".to_owned(),
            "-e".to_owned(),
            self.rpc.to_owned(),
            "--private-key".to_owned(),
            self.private_key.to_owned(),
        ];
        if let Some(args) = &self.constructor_args {
            if let Some(value) = &self.constructor_value {
                deploy_args.push("--constructor-value".to_owned());
                deploy_args.push(value.to_owned());
            }
            if let Some(deployer) = &self.stylus_deployer {
                deploy_args.push("--deployer-address".to_owned());
                deploy_args.push(deployer.to_string());
            }
            // Must add the args at the end
            deploy_args.push("--constructor-args".to_owned());
            deploy_args.extend_from_slice(args);
        }
        Ok(deploy_args)
    }
}

fn extract_deploy_result(s: &str) -> Result<(&str, &str, f64)> {
    let mut address = None;
    let mut tx_hash = None;
    let mut gas_estimate = None;
    for line in s.lines() {
        if let Some((_, rest)) = line.split_once("deployed code at address: ") {
            address = Some(rest);
        } else if let Some((_, rest)) = line.split_once("deployment tx hash: ") {
            tx_hash = Some(rest);
        } else if let Some(ge) = parse_gas_estimate(line) {
            gas_estimate = Some(ge)
        }
    }
    match (address, tx_hash, gas_estimate) {
        (Some(address), Some(tx_hash), Some(gas_estimate)) => Ok((address, tx_hash, gas_estimate)),
        _ => bail!("failed to extract deployed address and tx hash"),
    }
}

fn extract_gas_estimate_result(s: &str) -> Result<f64> {
    for line in s.lines() {
        if let Some(ge) = parse_gas_estimate(line) {
            return Ok(ge);
        }
    }
    bail!("failed to extract deployed address and tx hash")
}

fn parse_gas_estimate(s: &str) -> Option<f64> {
    let re: Regex =
        Regex::new(r"wasm data fee: (\d+\.\d+) ETH \(originally (\d+\.\d+) ETH with 20% bump\)")
            .unwrap();
    match re.captures(s) {
        Some(caps) => Result::ok(caps.get(2).map(|fee| fee.as_str().parse::<f64>()).unwrap()),
        None => None,
    }
}

pub fn call<I: IntoIterator<Item = String>>(
    dir: &Option<String>,
    func: &str,
    args: I,
) -> Result<String> {
    let original_dir = env::current_dir()?;
    if let Some(dir) = dir {
        env::set_current_dir(Path::new(&dir))?;
    }
    let output = Command::new("cargo")
        .arg("stylus-beta")
        .arg(func)
        .args(args)
        .output()
        .wrap_err(format!("failed to run cargo stylus {func}"))?;
    env::set_current_dir(original_dir)?;
    if !output.status.success() {
        let err = String::from_utf8(output.stderr)
            .map(strip_color)
            .unwrap_or("failed to decode error".to_owned());
        return Err(eyre!("failed to run node: {}", err));
    }
    String::from_utf8(output.stdout)
        .map(strip_color)
        .wrap_err("failed to decode stdout")
}

fn strip_color(s: impl Into<String>) -> String {
    let re = Regex::new(r"\x1b\[[0-9;]*[ABCDHJKSTfGmsu]").unwrap();
    re.replace_all(s.into().as_str(), "").into_owned()
}
