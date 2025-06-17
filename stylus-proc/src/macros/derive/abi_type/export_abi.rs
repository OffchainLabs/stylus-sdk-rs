// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use super::*;
use crate::imports::stylus_sdk::abi::{
    export::internal::{InnerType, InnerTypes},
    AbiType,
};
use proc_macro2::{Ident, Span};

pub struct ExportAbiExtension;

impl DeriveAbiTypeExtension for ExportAbiExtension {
    type Ast = syn::ItemImpl;

    /// When exporting the ABI, the code implements the InnerTypes trait.
    fn codegen(item: &syn::ItemStruct) -> syn::ItemImpl {
        let inner_types_out = Ident::new("out", Span::call_site());
        let sol_type = Ident::new("sol_type_str", Span::call_site());
        let ty_name = &item.ident;
        let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

        // Generate the code that append the inner types of each field
        let fields_inner_types: Vec<syn::Stmt> = item
            .fields
            .iter()
            .map(|field| {
                let ty = field.ty.clone();
                parse_quote! {
                    #inner_types_out.append(&mut <#ty as #InnerTypes>::inner_types());
                }
            })
            .collect();

        // Generate the code that writes each solidity field to the sol_type string
        let fields_sol_types: Vec<syn::Stmt> = item
            .fields
            .iter()
            .map(|field| {
                let ty = field.ty.clone();
                let name = field.ident.clone().map(|i| i.to_string()).unwrap_or_default();
                parse_quote! {
                    #sol_type.push_str(&format!("{}{};", <#ty as #AbiType>::ABI, underscore_if_sol(#name)));
                }
            })
            .collect();

        parse_quote! {
            impl #impl_generics #InnerTypes for #ty_name #ty_generics #where_clause {
                fn inner_types() -> alloc::vec::Vec<#InnerType> {
                    use alloc::{format, vec::Vec};
                    use core::any::TypeId;
                    use stylus_sdk::abi::export::underscore_if_sol;

                    let mut #inner_types_out: Vec<#InnerType> = Vec::new();
                    #(#fields_inner_types)*

                    let mut #sol_type = String::new();
                    #sol_type.push_str(&format!("struct {} {{", <#ty_name as #AbiType>::ABI));
                    #(#fields_sol_types)*
                    #sol_type.push_str(&format!("}}"));
                    let id = TypeId::of::<#ty_name>();
                    #inner_types_out.push(#InnerType { name: #sol_type, id });

                    #inner_types_out
                }
            }
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
