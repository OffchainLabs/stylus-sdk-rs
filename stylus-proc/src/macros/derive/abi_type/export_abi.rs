// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use super::*;
use crate::imports::stylus_sdk::abi::InnerTypes;

pub struct ExportAbiExtension;

impl DeriveAbiTypeExtension for ExportAbiExtension {
    type Ast = syn::ItemImpl;

    /// When exporting the ABI, the code implements the InnerTypes trait.
    fn codegen(item: &syn::ItemStruct) -> syn::ItemImpl {
        let name = &item.ident;
        let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

        parse_quote! {
            impl #impl_generics #InnerTypes for #name #ty_generics #where_clause {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DeriveAbiTypeExtension, ExportAbiExtension};
    use crate::utils::testing::assert_ast_eq;
    use syn::parse_quote;

    #[test]
    fn test_impl_inner_types_for_derive_abi() {
        let item: syn::ItemStruct = parse_quote! {
            struct Foo<T>
            where T: Bar {
                a: bool,
                b: String,
                t: T,
            }
        };
        assert_ast_eq(
            ExportAbiExtension::codegen(&item),
            parse_quote! {
                impl<T> stylus_sdk::abi::export::internal::InnerTypes for Foo<T>
                where T: Bar {
                }
            },
        )
    }
}
