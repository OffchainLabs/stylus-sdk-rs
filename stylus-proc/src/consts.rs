// Copyright 2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Constants for name definitions in generated code.
//!
//! Any generated globals or associated items should use a `__stylus` prefix to avoid name
//! collisions.

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;

/// Name of the entrypoint funciton that is generated for struct-based contracts.
pub const STRUCT_ENTRYPOINT_FN: ConstIdent = ConstIdent("__stylus_struct_entrypoint");

/// Name of the associated function that can be called to assert safe overrides at compile-time.
pub const ASSERT_OVERRIDES_FN: ConstIdent = ConstIdent("__stylus_assert_overrides");

/// Name of the associated function that can be called to check safe overriding of a single
/// function.
pub const ALLOW_OVERRIDE_FN: ConstIdent = ConstIdent("__stylus_allow_override");

/// Definition of a constant identifier
pub struct ConstIdent(&'static str);

impl ConstIdent {
    pub fn as_ident(&self) -> syn::Ident {
        syn::Ident::new(self.0, Span::mixed_site())
    }
}

impl ToTokens for ConstIdent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.as_ident().to_tokens(tokens);
    }
}
