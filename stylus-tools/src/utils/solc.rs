// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![allow(dead_code)]

use super::{color::Color, sys};

const LINK: &str = "https://docs.soliditylang.org/en/latest/installing-solidity.html";

pub fn check_exists() -> Result<(), SolcError> {
    if sys::command_exists("solc") {
        Ok(())
    } else {
        Err(SolcError::CommandDoesNotExist)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SolcError {
    #[error("solc not found. Please see\n{link}", link = LINK.red())]
    CommandDoesNotExist,
}
