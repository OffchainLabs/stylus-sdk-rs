// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! General purpose utilities.
//!
//! While none of these utilities have any funcitonality specific to Stylus, they are used by
//! [`stylus-tools`](crate) to operate on Stylus contracts and their associated artifacts.

// TODO: what do we want to expose here

use std::{fs, path::Path};

use alloy::primitives::U256;
use bytesize::ByteSize;
use color::{Color, GREY, MINT, PINK, YELLOW};

use crate::Result;

pub mod cargo;
pub mod color;
pub mod sys;

pub(crate) mod docker;
pub(crate) mod git;
pub(crate) mod solc;
pub(crate) mod stylus_sdk;
pub(crate) mod toolchain;
pub(crate) mod wasm;

/// Pretty-prints a data fee.
pub fn format_data_fee(fee: U256) -> String {
    // TODO: alternative to magic numbers
    let Ok(fee): Result<u64, _> = (fee / U256::from(1e9)).try_into() else {
        return "???".red();
    };
    let fee = fee as f64 / 1e9;
    let text = format!("{fee:.6} ETH");
    if fee <= 5e14 {
        text.mint()
    } else if fee <= 5e15 {
        text.yellow()
    } else {
        text.red()
    }
}

/// Pretty-prints a file size based on its limits.
pub fn format_file_size(len: ByteSize, mid: ByteSize, max: ByteSize) -> String {
    let color = if len <= mid {
        MINT
    } else if len <= max {
        YELLOW
    } else {
        PINK
    };

    format!("{color}{len}{GREY} ({} bytes)", len.as_u64())
}

/// Check if a directory exists, creating it if not.
pub fn create_dir_if_dne(path: impl AsRef<Path>) -> std::io::Result<()> {
    let path = path.as_ref();
    if !path.is_dir() {
        fs::create_dir(path)?;
    }
    Ok(())
}

pub fn sanitize_package_name(name: &str) -> String {
    name.replace('-', "_").replace('"', "")
}

pub fn bump_data_fee(data_fee: U256, bump_percent: u64) -> U256 {
    data_fee * U256::from(100 + bump_percent) / U256::from(100)
}
