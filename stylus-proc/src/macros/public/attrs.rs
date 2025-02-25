// Copyright 2022-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Token,
};

/// Inherit from parent contracts.
///
/// Used for the `#[inherit(Parent1, Parent2]` attribute.
pub struct Inherit {
    pub types: Punctuated<syn::Type, Token![,]>,
}

impl Parse for Inherit {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            types: Punctuated::parse_terminated(input)?,
        })
    }
}

/// Implement parent trait routes.
///
/// Used for the `#[implements(Parent1, Parent2]` attribute.
///
/// The contract must implement whatever traits are specified.
pub struct Implements {
    pub types: Punctuated<syn::Type, Token![,]>,
}

impl Parse for Implements {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            types: Punctuated::parse_terminated(input)?,
        })
    }
}

/// Selector name overloading for public functions.
///
/// Used for the `#[selector(name = "...")]` attribute.
#[derive(Debug)]
pub struct Selector {
    _name: kw::name,
    _eq_token: Token![=],
    pub value: syn::LitStr,
}

impl Parse for Selector {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _name: input.parse()?,
            _eq_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

mod kw {
    syn::custom_keyword!(name);
}
