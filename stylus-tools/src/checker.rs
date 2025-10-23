// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use crate::call;
use eyre::Result;
use typed_builder::TypedBuilder;

/// Defines the configuration for checking a Stylus contract.
/// After setting the parameters, call `Checker::check` to perform the check.
#[derive(TypedBuilder)]
#[builder(field_defaults(default, setter(into)))]
pub struct Checker {
    #[builder(!default)]
    rpc: String,

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
