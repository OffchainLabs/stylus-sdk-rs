// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

const TARGET: &str = "wasm32-unknown-unknown";
const OPT_LEVEL_Z: &str = "profile.release.opt-level='z'";
const UNSTABLE_FLAGS: &[&str] = &[
    "build-std=std,panic_abort",
    "build-std-features=panic_immediate_abort",
];

/// Build a Stylus project.
#[derive(Debug)]
pub struct StylusBuild {}
