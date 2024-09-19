// Copyright 2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Constants for referencing imports within generated code.
//!
//! These constants use a fully qualified path with dependencies nested within [`stylus_sdk`] to
//! ensure compatibility.
//!
//! Usage:
//! ```compile_fail
//! use crate::imports::alloy_primitives::Address;
//!
//! let _ = quote! {
//!     let addr = #Address::random();
//! };
//! ```

#![allow(non_upper_case_globals)]

use proc_macro2::TokenStream;
use quote::ToTokens;

pub mod alloy_primitives {
    use crate::imports::ConstPath;

    pub const Address: ConstPath = ConstPath("stylus_sdk::alloy_primitives::Address");
}

pub mod alloy_sol_types {
    use crate::imports::ConstPath;

    pub const SolType: ConstPath = ConstPath("stylus_sdk::alloy_sol_types::SolType");
    pub const SolValue: ConstPath = ConstPath("stylus_sdk::alloy_sol_types::SolValue");

    pub mod private {
        use crate::imports::ConstPath;

        pub const SolTypeValue: ConstPath =
            ConstPath("stylus_sdk::alloy_sol_types::private::SolTypeValue");
    }

    pub mod sol_data {
        use crate::imports::ConstPath;

        pub const Address: ConstPath = ConstPath("stylus_sdk::alloy_sol_types::sol_data::Address");
    }
}

pub mod stylus_sdk {
    pub mod abi {
        use crate::imports::ConstPath;

        pub const AbiType: ConstPath = ConstPath("stylus_sdk::abi::AbiType");
    }
}

/// Definition of a fully-qualified path for generated code.
pub struct ConstPath(&'static str);

impl ConstPath {
    /// Interpret the path as a [`syn::Type`].
    pub fn as_type(&self) -> syn::Type {
        syn::parse_str(self.0).unwrap()
    }
}

impl ToTokens for ConstPath {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let path: syn::Path = syn::parse_str(self.0).unwrap();
        path.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    #[test]
    fn test_const_path() {
        assert_eq!(
            super::alloy_primitives::Address.as_type(),
            parse_quote!(stylus_sdk::alloy_primitives::Address),
        );
    }
}
