// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use crate::call;
use eyre::{bail, Result};

/// Defines the configuration for activating a Stylus contract.
/// After setting the parameters, call `Activator::activate` to perform the activation.
pub struct Activator {
    rpc: String,
    dir: Option<String>,
    private_key: Option<String>,
    contract_address: Option<String>,
}

impl Activator {
    // Create the Activator with default parameters.
    pub fn new(rpc: String) -> Self {
        cfg_if::cfg_if! {
            // When running with integration tests, set the default parameters for the local devnet.
            if #[cfg(feature = "integration-tests")] {
                Self {
                    rpc,
                    dir: None,
                    private_key: Some(crate::devnet::DEVNET_PRIVATE_KEY.to_owned()),
                    contract_address: None,
                }
            } else {
                Self {
                    rpc,
                    dir: None,
                    private_key: None,
                    contract_address: None,
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

    pub fn with_contract_address(mut self, address: String) -> Self {
        self.contract_address = Some(address);
        self
    }

    // Activate the Stylus contract.
    pub fn activate(&self) -> Result<()> {
        let Some(private_key) = &self.private_key else {
            bail!("missing private key");
        };
        let Some(contract_address) = &self.contract_address else {
            bail!("missing contract address");
        };
        let activate_args = vec![
            "-e".to_owned(),
            self.rpc.to_owned(),
            "--private-key".to_owned(),
            private_key.to_owned(),
            "--address".to_owned(),
            contract_address.to_owned(),
        ];

        let res = call(&self.dir, "activate", activate_args);
        match res {
            Ok(_) => Ok(()),
            Err(err) => {
                if err.to_string().contains("ProgramUpToDate()") {
                    Ok(())
                } else {
                    Err(err)
                }
            }
        }
    }
}
