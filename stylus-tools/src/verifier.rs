// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use crate::call;
use eyre::{bail, Result};

/// Defines the configuration for verifying a Stylus contract.
/// After setting the parameters, call `Verifier::verify` to perform the verification.
pub struct Verifier {
    rpc: String,
    dir: Option<String>,
    deployment_tx_hash: Option<String>,
}

impl Verifier {
    // Create the Deployer with default parameters.
    pub fn new(rpc: String) -> Self {
        Self {
            rpc,
            dir: None,
            deployment_tx_hash: None,
        }
    }

    pub fn with_contract_dir(mut self, dir: String) -> Self {
        self.dir = Some(dir);
        self
    }

    pub fn with_deployment_tx_hash(mut self, value: String) -> Self {
        self.deployment_tx_hash = Some(value);
        self
    }

    // Verify the deployed Stylus contract.
    pub fn verify(self) -> Result<()> {
        let Some(deployment_tx_hash) = self.deployment_tx_hash else {
            bail!("missing deployment tx hash");
        };
        let res = call(
            &self.dir,
            "verify",
            vec![
                "--no-verify".to_owned(),
                "-e".to_owned(),
                self.rpc,
                "--deployment-tx".to_owned(),
                deployment_tx_hash,
            ],
        );

        match res {
            Err(e) => {
                if e.to_string().contains("not yet implemented") {
                    Ok(())
                } else {
                    Err(e)
                }
            }
            Ok(_) => Ok(()),
        }
    }
}
