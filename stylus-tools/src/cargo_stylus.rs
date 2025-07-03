// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::primitives::Address;
use eyre::{bail, eyre, Result, WrapErr};
use regex::Regex;
use std::{ffi::OsStr, process::Command, env, path::Path};

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
    pub fn deploy(self) -> Result<Address> {
        let mut deploy_args: Vec<String> = Vec::new();
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

        let original_dir = env::current_dir()?;
        if let Some(dir) = self.dir {
            env::set_current_dir(Path::new(&dir))?;
        }
        let res = call_deploy(deploy_args);
        env::set_current_dir(original_dir)?;
        return res;
    }
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
