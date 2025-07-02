// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::fmt::Display;

use eyre::bail;

use style::{BOLD, ERROR};

mod style;

pub fn convert_gwei_to_wei(fee_str: &str) -> eyre::Result<u128> {
    let gwei = match fee_str.parse::<f64>() {
        Ok(fee) if fee >= 0.0 => fee,
        Ok(_) => bail!("Max fee per gas must be non-negative"),
        Err(_) => bail!("Invalid max fee per gas value: {}", fee_str),
    };

    if !gwei.is_finite() {
        bail!("Invalid gwei value: must be finite");
    }

    let wei = gwei * 1e9;
    if !wei.is_finite() {
        bail!("Overflow occurred in floating point multiplication of --max-fee-per-gas-gwei converting");
    }

    if wei < 0.0 || wei >= u128::MAX as f64 {
        bail!("Result outside valid range for wei");
    }

    Ok(wei as u128)
}

pub fn decode0x(text: impl AsRef<str>) -> eyre::Result<Vec<u8>> {
    let text = text.as_ref();
    let text = text.trim();
    let text = text.strip_prefix("0x").unwrap_or(text);
    Ok(hex::decode(text)?)
}

pub fn print_error(err: impl Display) {
    eprintln!("{ERROR}error{ERROR:#}{BOLD}:{BOLD:#} {err}");
}
