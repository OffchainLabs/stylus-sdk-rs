// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use crate::call;
use derive_builder::Builder;
use eyre::Result;

/// Defines the configuration for activating a Stylus contract.
/// After setting the parameters, call `Activator::activate` to perform the activation.
#[derive(Builder)]
#[builder(setter(into))]
pub struct Activator {
    rpc: String,

    #[builder(default)]
    dir: Option<String>,

    #[cfg_attr(
        feature = "integration-tests",
        builder(default = "crate::devnet::DEVNET_PRIVATE_KEY.to_owned()")
    )]
    private_key: String,

    contract_address: String,
}

impl Activator {
    // Activate the Stylus contract.
    pub fn activate(&self) -> Result<()> {
        let activate_args = vec![
            "-e".to_owned(),
            self.rpc.to_owned(),
            "--private-key".to_owned(),
            self.private_key.to_owned(),
            "--address".to_owned(),
            self.contract_address.to_owned(),
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
