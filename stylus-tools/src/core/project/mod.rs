// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

pub mod hash;

pub type ProjectHash = [u8; 32];

#[derive(Debug)]
pub enum ProjectKind {
    Contract,
    Workspace,
}
