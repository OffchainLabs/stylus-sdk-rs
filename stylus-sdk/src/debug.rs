// Copyright 2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use crate::hostio;

/// Prints a UTF-8 encoded string to the console. Only available in debug mode.
pub fn console_log<T: AsRef<str>>(text: T) {
    let text = text.as_ref();
    unsafe { hostio::log_txt(text.as_ptr(), text.len()) };
}

/// Prints to the console when executing in a debug environment. Otherwise does nothing.
#[cfg(feature = "debug")]
#[macro_export]
macro_rules! console {
    ($($msg:tt)*) => {
        $crate::debug::console_log(format!($($msg)*));
    };
}

/// Prints to the console when executing in a debug environment. Otherwise does nothing.
#[cfg(not(feature = "debug"))]
#[macro_export]
macro_rules! console {
    ($($msg:tt)*) => {
        $crate::debug::console_log(format!($($msg)*));
    };
}
