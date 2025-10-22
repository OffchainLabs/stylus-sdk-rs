// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use crate::call;
use derive_builder::Builder;
use eyre::Result;

/// Defines the configuration for checking a Stylus contract.
/// After setting the parameters, call `Checker::check` to perform the check.
#[derive(Builder)]
#[builder(setter(into))]
pub struct Checker {
    rpc: String,

    #[builder(default)]
    dir: Option<String>,
}

impl Checker {
    // Checks the Stylus contract.
    pub fn check(self) -> Result<()> {
        let check_args: Vec<String> = vec!["-e".to_owned(), self.rpc, "--verbose".to_owned()];

        call(&self.dir, "check", check_args)?;

        Ok(())
    }
}
