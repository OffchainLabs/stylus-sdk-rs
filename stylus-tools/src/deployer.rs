// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::primitives::{Address, TxHash};
use eyre::{bail, eyre, Result, WrapErr};
use regex::Regex;
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

    // Deploy the Stylus contract returning the contract address.
    pub fn deploy(self) -> Result<(Address, TxHash)> {
        let mut deploy_args: Vec<String> = Vec::new();
        deploy_args.push("--no-verify".to_owned());
        deploy_args.push("-e".to_owned());
        deploy_args.push(self.rpc);
        let Some(private_key) = self.private_key else {
            bail!("missing private key");
        };
        deploy_args.push("--private-key".to_owned());
        deploy_args.push(private_key);
        if let Some(args) = self.constructor_args {
            if let Some(value) = self.constructor_value {
                deploy_args.push("--constructor-value".to_owned());
                deploy_args.push(value);
            }
            if let Some(deployer) = self.stylus_deployer {
                deploy_args.push("--deployer-address".to_owned());
                deploy_args.push(deployer.to_string());
            }
            // Must add the args at the end
            deploy_args.push("--constructor-args".to_owned());
            deploy_args.extend_from_slice(&args);
        }

        let out = call(self.dir, "deploy", deploy_args)?;
        let out = strip_color(&out);
        let (address, tx_hash) = extract_deployed_address(&out)?;
        let address: Address = address
            .parse()
            .wrap_err("failed to parse deployment address")?;
        let tx_hash: TxHash = tx_hash
            .parse()
            .wrap_err("failed to parse deployment tx hash")?;
        Ok((address, tx_hash))
    }
}

pub fn call<I: IntoIterator<Item = String>>(
    dir: Option<String>,
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

fn strip_color(s: &str) -> String {
    let re = Regex::new(r"\x1b\[[0-9;]*[ABCDHJKSTfGmsu]").unwrap();
    re.replace_all(s, "").into_owned()
}

fn extract_deployed_address(s: &str) -> Result<(&str, &str)> {
    let mut address = None;
    let mut tx_hash = None;
    for line in s.lines() {
        if let Some((_, rest)) = line.split_once("deployed code at address: ") {
            address = Some(rest);
        } else if let Some((_, rest)) = line.split_once("deployment tx hash: ") {
            tx_hash = Some(rest);
        }
    }
    Option::zip(address, tx_hash).ok_or_else(|| eyre!("deployment address or tx hash not found"))
}
