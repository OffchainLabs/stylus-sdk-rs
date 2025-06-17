// Copyright 2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Constants for name definitions in generated code.
//!
//! Any generated globals or associated items should use a `__stylus` prefix to avoid name
//! collisions.

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;

/// Name of the entrypoint function that is generated for struct-based contracts.
pub const STRUCT_ENTRYPOINT_FN: ConstIdent = ConstIdent("__stylus_struct_entrypoint");

pub const STYLUS_HOST_FIELD: ConstIdent = ConstIdent("__stylus_host");

pub const STYLUS_CONTRACT_ADDRESS_FIELD: ConstIdent = ConstIdent("__stylus_contract_address");

/// Definition of a constant identifier
pub struct ConstIdent(&'static str);

impl ConstIdent {
    pub fn as_ident(&self) -> syn::Ident {
        syn::Ident::new(self.0, Span::call_site())
    }
}

impl ToTokens for ConstIdent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.as_ident().to_tokens(tokens);
    }
}
