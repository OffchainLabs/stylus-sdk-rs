// Copyright 2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Macro generation utilities.

use sha3::{Digest, Keccak256};
use syn::{punctuated::Punctuated, Token};
use syn_solidity::SolIdent;

pub mod attrs;

#[cfg(test)]
pub mod testing;

pub fn get_generics(
    generics: &syn::Generics,
) -> (
    Punctuated<syn::GenericParam, Token![,]>,
    Punctuated<syn::WherePredicate, Token![,]>,
) {
    let generic_params = generics.params.clone();
    let where_clause = generics
        .where_clause
        .clone()
        .map(|c| c.predicates)
        .unwrap_or_default();
    (generic_params, where_clause)
}

/// Build [function selector](https://solidity-by-example.org/function-selector/) byte array.
pub fn build_selector<'a>(
    name: &SolIdent,
    params: impl Iterator<Item = &'a syn_solidity::Type>,
) -> [u8; 4] {
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
