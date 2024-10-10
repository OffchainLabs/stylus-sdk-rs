// Copyright 2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Utilities for testing.

use quote::ToTokens;

/// Assert equality of two AST nodes, with pretty diff output for failures.
pub fn assert_ast_eq<T: ToTokens>(left: T, right: T) {
    let left = pprint(left);
    let right = pprint(right);
    pretty_assertions::assert_str_eq!(left, right);
}

fn pprint<T: ToTokens>(node: T) -> String {
    let tokens = node.into_token_stream();
    let file = syn::parse2(tokens).unwrap();
    prettyplease::unparse(&file)
}
