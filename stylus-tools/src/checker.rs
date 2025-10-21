// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use crate::call;
use eyre::Result;

/// Defines the configuration for checking a Stylus contract.
/// After setting the parameters, call `Checker::check` to perform the check.
pub struct Checker {
    rpc: String,
    dir: Option<String>,
}

impl Checker {
    // Create the Checker with default parameters.
    pub fn new(rpc: String) -> Self {
        Self { rpc, dir: None }
    }

    pub fn with_contract_dir(mut self, dir: String) -> Self {
        self.dir = Some(dir);
        self
    }

    // Checks the Stylus contract.
    pub fn check(self) -> Result<()> {
        let check_args: Vec<String> = vec!["-e".to_owned(), self.rpc, "--verbose".to_owned()];

        call(&self.dir, "check", check_args)?;

        Ok(())
    }
}
