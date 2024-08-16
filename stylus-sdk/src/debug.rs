// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

//! Debug-only items for printing to the console.
//!
//! ```no_run
//! use stylus_sdk::console;
//! use stylus_sdk::alloy_primitives::address;
//! extern crate alloc;
//!
//! let arbinaut = address!("361594F5429D23ECE0A88E4fBE529E1c49D524d8");
//! console!("Gm {}", arbinaut); // prints nothing in production
//! ```

/// Prints a UTF-8 encoded string to the console. Only available in debug mode.
#[cfg(feature = "debug")]
pub fn console_log<T: AsRef<str>>(text: T) {
    let text = text.as_ref();
    unsafe { crate::hostio::log_txt(text.as_ptr(), text.len()) };
}

/// Prints to the console when executing in a debug environment. Otherwise does nothing.
#[cfg(feature = "debug")]
#[macro_export]
macro_rules! console {
    ($($msg:tt)*) => {
        $crate::debug::console_log(alloc::format!($($msg)*));
    };
}

/// Prints to the console when executing in a debug environment. Otherwise does nothing.
#[cfg(not(feature = "debug"))]
#[macro_export]
macro_rules! console {
    ($($msg:tt)*) => {{}};
}
