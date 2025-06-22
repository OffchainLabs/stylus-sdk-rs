// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::fmt::Display;

use style::{BOLD, ERROR};

mod style;

pub fn decode0x<T: AsRef<str>>(text: T) -> eyre::Result<Vec<u8>> {
    let text = text.as_ref();
    let text = text.trim();
    let text = text.strip_prefix("0x").unwrap_or(text);
    Ok(hex::decode(text)?)
}

pub fn print_error(err: impl Display) {
    eprintln!("{ERROR}error{ERROR:#}{BOLD}:{BOLD:#} {err}");
}
