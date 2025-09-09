// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::json_abi::Constructor;

use crate::core::reflection::{reflect, ReflectionError};

pub fn constructor() -> Result<Option<Constructor>, ReflectionError> {
    greyln!("checking whether the contract has a constructor...");
    let output = reflect("constructor", vec![])?;
    let output = std::str::from_utf8(&output)?;
    parse_constructor(output)
}

fn parse_constructor(signature: &str) -> Result<Option<Constructor>, ReflectionError> {
    let signature = signature.trim();
    if !signature.starts_with("constructor") {
        // If the signature doesn't start with constructor, it is either an old SDK version that
        // doesn't support it or the contract doesn't have one. So, it is safe to return None.
        Ok(None)
    } else {
        Constructor::parse(signature).map(Some).map_err(Into::into)
    }
}
