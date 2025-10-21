// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::primitives::{Address, TxHash};
use eyre::{bail, Result, WrapErr};
use regex::Regex;
use std::iter::once;
use std::{env, path::Path, process::Command};

/// Defines the configuration for deploying a Stylus contract.
/// After setting the parameters, call `Deployer::deploy` to perform the deployement.
pub struct Deployer {
    rpc: String,
    dir: Option<String>,
    private_key: Option<String>,
    stylus_deployer: Option<String>,
    constructor_value: Option<String>,
    constructor_args: Option<Vec<String>>,
}

impl Deployer {
    // Create the Deployer with default parameters.
    pub fn new(rpc: String) -> Self {
        cfg_if::cfg_if! {
            // When running with integration tests, set the default parameters for the local devnet.
            if #[cfg(feature = "integration-tests")] {
                Self {
                    rpc,
                    dir: None,
                    private_key: Some(crate::devnet::DEVNET_PRIVATE_KEY.to_owned()),
                    stylus_deployer: Some(crate::devnet::addresses::STYLUS_DEPLOYER.to_string()),
                    constructor_value: None,
                    constructor_args: None,
                }
            } else {
                Self {
                    rpc,
                    dir: None,
                    private_key: None,
                    stylus_deployer: None,
                    constructor_value: None,
                    constructor_args: None,
                }
            }
        }
    }

    pub fn with_contract_dir(mut self, dir: String) -> Self {
        self.dir = Some(dir);
        self
    }

    pub fn with_private_key(mut self, key: String) -> Self {
        self.private_key = Some(key);
        self
    }

    pub fn with_stylus_deployer(mut self, deployer: Address) -> Self {
        self.stylus_deployer = Some(deployer.to_string());
        self
    }

    pub fn with_constructor_value(mut self, value: String) -> Self {
        self.constructor_value = Some(value);
        self
    }

    pub fn with_constructor_args(mut self, args: Vec<String>) -> Self {
        self.constructor_args = Some(args);
        self
    }

    pub fn estimate_gas(&self) -> Result<f64> {
        let deploy_args = self.deploy_args()?;
        let out = call(
            &self.dir,
            "deploy",
            once("--estimate-gas".to_owned()).chain(deploy_args),
        )?;
        let out = strip_color(&out);

        extract_gas_estimate(&out)
    }

    // Deploy the Stylus contract returning the contract address.
    pub fn deploy(&self) -> Result<(Address, TxHash, f64)> {
        let deploy_args = self.deploy_args()?;

        let out = call(&self.dir, "deploy", deploy_args)?;
        let out = strip_color(&out);
        let (address, tx_hash, gas_estimate) = extract_deployed_address(&out)?;
        let address: Address = address
            .parse()
            .wrap_err("failed to parse deployment address")?;
        let tx_hash: TxHash = tx_hash
            .parse()
            .wrap_err("failed to parse deployment tx hash")?;
        Ok((address, tx_hash, gas_estimate))
    }

    fn deploy_args(&self) -> Result<Vec<String>> {
        let mut deploy_args: Vec<String> = Vec::new();
        deploy_args.push("--no-verify".to_owned());
        deploy_args.push("-e".to_owned());
        deploy_args.push(self.rpc.to_owned());
        let Some(private_key) = &self.private_key else {
            bail!("missing private key");
        };
        deploy_args.push("--private-key".to_owned());
        deploy_args.push(private_key.to_owned());
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

fn strip_color(s: &str) -> String {
    let re = Regex::new(r"\x1b\[[0-9;]*[ABCDHJKSTfGmsu]").unwrap();
    re.replace_all(s, "").into_owned()
}

fn extract_deployed_address(s: &str) -> Result<(&str, &str, f64)> {
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

fn extract_gas_estimate(s: &str) -> Result<f64> {
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
        let err = String::from_utf8(output.stderr).unwrap_or("failed to decode error".to_owned());
        bail!("failed to run node: {}", err);
    }
    String::from_utf8(output.stdout).wrap_err("failed to decode stdout")
}
