// Copyright 2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Fields, ItemEnum};

pub fn derive_solidity_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemEnum);
    let name = &input.ident;
    let mut match_arms = quote!();
    let mut errors = vec![];
    for variant in input.variants {
        let variant_name = variant.ident;
        let error = match variant.fields {
            Fields::Unnamed(e) if variant.fields.len() == 1 => e.unnamed.first().unwrap().clone(),
            _ => error!(variant.fields, "Variant not a 1-tuple"),
        };
        match_arms.extend(quote! {
            #name::#variant_name(e) => e.encode(),
        });
        errors.push(error);
    }
    quote! {
        impl From<#name> for ::alloc::vec::Vec<u8> {
            fn from(err: #name) -> ::alloc::vec::Vec<u8> {
                match err {
                    #match_arms
                }
            }
        }

        impl stylus_sdk::abi::export::internal::InnerTypes for #name {
            fn inner_types() -> Vec<stylus_sdk::abi::export::internal::InnerType> {
                use alloc::vec;
                use core::any::TypeId;
                use stylus_sdk::abi::export::internal::InnerType;
                use stylus_sdk::alloy_sol_types::SolError;

                vec![
                    #(
                        InnerType {
                            name: format!("error {};", <#errors as SolError>::SIGNATURE.replace(',', ", ")),
                            id: TypeId::of::<#errors>(),
                        }
                    ),*
                ]
            }
        }
    }.into()
}
