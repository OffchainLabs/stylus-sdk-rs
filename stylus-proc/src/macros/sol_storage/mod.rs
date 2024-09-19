// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, punctuated::Punctuated, Token};

use proc::{SolidityField, SolidityFields, SolidityStruct, SolidityStructs};

mod proc;

pub fn sol_storage(input: TokenStream) -> TokenStream {
    let SolidityStructs(decls) = parse_macro_input!(input as SolidityStructs);
    let mut out = quote!();

    for decl in decls {
        let SolidityStruct {
            attrs,
            vis,
            name,
            generics,
            fields: SolidityFields(fields),
        } = decl;

        let fields: Punctuated<_, Token![,]> = fields
            .into_iter()
            .map(|SolidityField { attrs, name, ty }| -> syn::Field {
                parse_quote! {
                    #(#attrs)*
                    pub #name: #ty
                }
            })
            .collect();

        out.extend(quote! {
            #(#attrs)*
            #[stylus_sdk::stylus_proc::storage]
            #vis struct #name #generics {
                #fields
            }
        });
    }

    out.into()
}
