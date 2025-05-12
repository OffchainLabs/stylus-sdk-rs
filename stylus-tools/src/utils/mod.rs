// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! General purpose utilities.
//!
//! While none of these utilities have any funcitonality specific to Stylus, they are used by
//! [`stylus-tools`](crate) to operate on Stylus contracts and their associated artifacts.

// TODO: what do we want to expose here

use bytesize::ByteSize;
use color::{GREY, MINT, PINK, YELLOW};

pub(crate) mod color;
pub(crate) mod docker;
pub(crate) mod wasm;

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
