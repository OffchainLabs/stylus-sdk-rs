// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

//! Types relating to method definitions.

/// State mutability of a contract function. This is currently used for checking whether contracts
/// are allowed to override a function from another contract they inherit from.
/// Users should not need this type outside of proc macros.
#[derive(Debug, Clone, Copy)]
pub enum Purity {
    /// No state read/write.
    Pure,
    /// No state write.
    View,
    /// Cannot receive Ether.
    Write,
    /// Everything is allowed.
    Payable,
}

impl Purity {
    /// Returns whether a function defined with this purity may be overridden
    /// by one with the given purity.
    pub const fn allow_override(&self, other: Purity) -> bool {
        use Purity::*;
        matches!(
            (*self, other),
            (Payable, Payable)
                | (Write, Write)
                | (Write, View)
                | (Write, Pure)
                | (View, View)
                | (View, Pure)
                | (Pure, Pure)
        )
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_allow_override() {
        use super::Purity::*;
        assert!(Payable.allow_override(Payable));
        assert!(!Payable.allow_override(Write));
        assert!(!Payable.allow_override(View));
        assert!(!Payable.allow_override(Pure));

        assert!(!Write.allow_override(Payable));
        assert!(Write.allow_override(Write));
        assert!(Write.allow_override(View));
        assert!(Write.allow_override(Pure));

        assert!(!View.allow_override(Payable));
        assert!(!View.allow_override(Write));
        assert!(View.allow_override(View));
        assert!(View.allow_override(Pure));

        assert!(!Pure.allow_override(Payable));
        assert!(!Pure.allow_override(Write));
        assert!(!Pure.allow_override(View));
        assert!(Pure.allow_override(Pure));
    }
}
