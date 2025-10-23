// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use crate::call;
use derive_builder::Builder;
use eyre::{eyre, Result};

/// Defines the configuration for verifying a Stylus contract.
/// After setting the parameters, call `Verifier::verify` to perform the verification.
#[derive(Builder)]
#[builder(setter(into))]
pub struct Verifier {
    rpc: String,

    #[builder(default)]
    dir: Option<String>,
    deployment_tx_hash: String,
}

impl Verifier {
    // Verify the deployed Stylus contract.
    pub fn verify(self) -> Result<()> {
        let verify_args = vec![
            "--no-verify".to_owned(),
            "-e".to_owned(),
            self.rpc,
            "--deployment-tx".to_owned(),
            self.deployment_tx_hash,
        ];
        call(&self.dir, "verify", verify_args).and_then(|out| {
            if out.contains("Verification successful") {
                Ok(())
            } else {
                Err(eyre!(out))
            }
        })
    }
}
