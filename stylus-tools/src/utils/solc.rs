// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![allow(dead_code)]

use super::color::Color;

const LINK: &str = "https://docs.soliditylang.org/en/latest/installing-solidity.html";

pub fn abi() -> Result<Vec<u8>, SolcError> {
    todo!()
}

pub fn check_exists() -> Result<(), SolcError> {
    todo!()
}

#[derive(Debug, thiserror::Error)]
pub enum SolcError {
    #[error("solc not found. Please see\n{link}", link = LINK.red())]
    CommandDoesNotExist,
}
