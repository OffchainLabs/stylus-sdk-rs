// Copyright 2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{parse_macro_input, parse_quote};

use crate::consts::STYLUS_HOST_FIELD;

/// Implementation of the [`#[derive(Erase)]`][crate::derive_erase] macro.
pub fn derive_erase(input: TokenStream) -> TokenStream {
    let node = parse_macro_input!(input as syn::ItemStruct);
    impl_erase(&node).into_token_stream().into()
}

/// Implement [`stylus_sdk::storage::Erase`] for the given struct.
///
/// Calls `Erase::erase()` on each of the members of the struct.
fn impl_erase(node: &syn::ItemStruct) -> syn::ItemImpl {
    let name = &node.ident;
    let (impl_generics, ty_generics, where_clause) = node.generics.split_for_impl();
    let filtered_fields = node
        .fields
        .clone()
        .into_iter()
        .filter_map(|field| match field.ident {
            Some(ident) if ident == STYLUS_HOST_FIELD.as_ident() => None,
            _ => field.ident,
        });

    parse_quote! {
        #[cfg(not(feature = "contract-client-gen"))]
        impl #impl_generics stylus_sdk::storage::Erase for #name #ty_generics #where_clause {
            fn erase(&mut self) {
                #(
                    self.#filtered_fields.erase();
                )*
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;
    use crate::utils::testing::assert_ast_eq;

    #[test]
    fn test_impl_erase() {
        assert_ast_eq(
            impl_erase(&parse_quote! {
                struct Foo<T: Erase> {
                    field1: StorageString,
                    field2: T,
                }
            }),
            parse_quote! {
                impl<T: Erase> stylus_sdk::storage::Erase for Foo<T> {
                    fn erase(&mut self) {
                        self.field1.erase();
                        self.field2.erase();
                    }
                }
            },
        );
    }
}
