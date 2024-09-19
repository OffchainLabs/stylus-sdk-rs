// Copyright 2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Utilities for handling macro attributes.

use proc_macro2::TokenStream;
use proc_macro_error::emit_error;
use syn::parse::{Nothing, Parse};

/// Consume any used attributes, leaving unused attributes in the list.
pub fn consume_attr<T: Parse>(
    attrs: &mut Vec<syn::Attribute>,
    ident_str: &'static str,
) -> Option<T> {
    let mut result = None;
    for attr in core::mem::take(attrs) {
        // skip all other attrs, adding them back to the Vec
        if !attr_ident_matches(&attr, ident_str) {
            attrs.push(attr);
            continue;
        }

        if result.is_some() {
            emit_error!(attr, "duplicate attribute");
        }

        let tokens = get_attr_tokens(&attr).unwrap_or_default();
        match syn::parse2(tokens) {
            Ok(value) => result = Some(value),
            Err(err) => {
                emit_error!(err.span(), "{}", err);
            }
        }
    }
    result
}

/// Consume a flag attribute (no input tokens)
pub fn consume_flag(attrs: &mut Vec<syn::Attribute>, ident_str: &'static str) -> bool {
    consume_attr::<Nothing>(attrs, ident_str).is_some()
}

/// Check that an attribute stream is empty.
pub fn check_attr_is_empty(attr: impl Into<TokenStream>) {
    let attr = attr.into();
    if let Err(err) = syn::parse2::<Nothing>(attr) {
        emit_error!(err.span(), "{}", err);
    }
}

/// Check if attribute is a simple [`syn::Ident`] and matches a given string
fn attr_ident_matches(attr: &syn::Attribute, value: &'static str) -> bool {
    matches!(attr.path().get_ident(), Some(ident) if *ident == value)
}

/// Get tokens for parsing from a [`syn::Attribute`].
///
/// We currently only need support for parenthesis as delimeters.
fn get_attr_tokens(attr: &syn::Attribute) -> Option<TokenStream> {
    if let syn::Meta::List(syn::MetaList {
        delimiter: syn::MacroDelimiter::Paren(_),
        tokens,
        ..
    }) = &attr.meta
    {
        Some(tokens.clone())
    } else {
        None
    }
}
