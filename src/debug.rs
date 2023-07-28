// Copyright 2023, Offchain Labs, Inc.
// For license information, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use crate::hostio;

/// Prints an encoded string to the console. Only available in debug mode.
pub fn println<T: AsRef<str>>(text: T) {
    let text = text.as_ref();
    unsafe { hostio::log_txt(text.as_ptr(), text.len()) };
}
