// Copyright 2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Macro generation utilities.

use std::borrow::Cow;

use sha3::{Digest, Keccak256};
use syn::{punctuated::Punctuated, Token};
use syn_solidity::SolIdent;

pub mod attrs;

#[cfg(test)]
pub mod testing;

/// Like [`syn::Generics::split_for_impl`] but for [`syn::ItemImpl`].
///
/// [`syn::Generics::split_for_impl`] does not work in this case because the `name` of the
/// implemented type is not easy to get, but the type including generics is.
pub fn split_item_impl_for_impl(
    node: &syn::ItemImpl,
) -> (
    Punctuated<syn::GenericParam, Token![,]>,
    syn::Type,
    Punctuated<syn::WherePredicate, Token![,]>,
) {
    let generic_params = node.generics.params.clone();
    let self_ty = (*node.self_ty).clone();
    let where_clause = node
        .generics
        .where_clause
        .clone()
        .map(|c| c.predicates)
        .unwrap_or_default();
    (generic_params, self_ty, where_clause)
}

/// Build [function selector](https://solidity-by-example.org/function-selector/) byte array.
pub fn build_selector(name: &SolIdent, params: impl Iterator<Item = Cow<'static, str>>) -> [u8; 4] {
    let mut selector = Keccak256::new();
    selector.update(name.to_string());
    selector.update("(");
    for (i, param) in params.enumerate() {
        if i > 0 {
            selector.update(",");
        }
        selector.update(param.to_string());
    }
    selector.update(")");
    let selector = selector.finalize();
    [selector[0], selector[1], selector[2], selector[3]]
}
