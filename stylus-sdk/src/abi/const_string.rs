// Copyright 2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use std::{
    fmt::{Debug, Display},
    ops::Deref,
};

pub const MAX_CONST_STRING_LENGTH: usize = 1024;

#[derive(Clone)]
pub struct ConstString {
    /// The signature's text encoding. Must be valid UTF-8.
    /// Note: this representation allows something approximating string manipulation to be const.
    data: [u8; MAX_CONST_STRING_LENGTH],
    len: usize,
}

/// Copies data from `source` to `dest` in a `const` context.
/// This function is very inefficient for other purposes.
const fn memcpy<const N: usize>(
    mut source: &[u8],
    mut dest: [u8; N],
    mut offset: usize,
) -> [u8; N] {
    if offset > dest.len() {
        panic!("out-of-bounds memcpy");
    }
    while !source.is_empty() {
        dest[offset] = source[0];
        offset += 1;
        (_, source) = source.split_at(1);
    }
    dest
}

impl ConstString {
    /// Creates a new [`ConstString`] equivalent to the empty string.
    pub const fn new(s: &str) -> ConstString {
        let mut data = [0u8; MAX_CONST_STRING_LENGTH];
        data = memcpy(s.as_bytes(), data, 0);
        ConstString { data, len: s.len() }
    }

    /// Creates a new [`ConstString`] from a decimal number.
    /// For example, the number 42 maps to "42".
    pub const fn from_decimal_number(mut number: usize) -> ConstString {
        let mut data = [0u8; MAX_CONST_STRING_LENGTH];
        let digits = number.checked_ilog10();
        let digits = match digits {
            // TODO: simplify when `const_precise_live_drops` is stabilized
            Some(digits) => digits as usize + 1,
            None => 1,
        };

        if digits > MAX_CONST_STRING_LENGTH {
            panic!("from_decimal_number: too many digits");
        }
        let mut position = digits;
        while position > 0 {
            position -= 1;
            data[position] = b'0' + (number % 10) as u8;
            number /= 10;
        }
        ConstString { data, len: digits }
    }

    /// Clones a [`ConstString`] in a `const` context.
    pub const fn const_clone(&self) -> Self {
        Self {
            data: self.data,
            len: self.len,
        }
    }

    /// Concatenates two [`ConstString`]'s.
    pub const fn concat(&self, other: ConstString) -> ConstString {
        let mut new = self.const_clone();
        new.data = memcpy(other.as_bytes(), new.data, self.len);
        new.len += other.len;
        new
    }

    /// Converts a [`ConstString`] to a slice.
    pub const fn as_bytes(&self) -> &[u8] {
        self.data.split_at(self.len).0
    }

    /// Converts a [`ConstString`] to an equivalent [`str`].
    pub const fn as_str(&self) -> &str {
        // # Safety
        // A `ConstString` represents a valid, utf8-encoded string
        unsafe { std::str::from_utf8_unchecked(self.as_bytes()) }
    }
}

impl Deref for ConstString {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}

impl Display for ConstString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Debug for ConstString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

#[test]
fn test_from_decimal() {
    for i in (0..=100).chain(1000..=1001) {
        assert_eq!(ConstString::from_decimal_number(i).as_str(), i.to_string());
    }
}