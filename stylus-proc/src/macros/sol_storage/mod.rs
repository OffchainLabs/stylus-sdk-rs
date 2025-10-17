// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use proc::{SolidityField, SolidityFields, SolidityStruct, SolidityStructs};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, punctuated::Punctuated, Token};

mod proc;

pub fn sol_storage(input: TokenStream) -> TokenStream {
    let SolidityStructs(decls) = parse_macro_input!(input as SolidityStructs);
    let mut out = quote!();

    for decl in decls {
        let SolidityStruct {
            attrs,
            vis,
            name,
            mut generics,
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

        generics
            .type_params_mut()
            .for_each(|ident| ident.bounds.push(parse_quote!(Default)));
        let (_, ty_generics, where_clause) = generics.split_for_impl();

        let is_entrypoint = attrs.iter().any(|attr| attr.path().is_ident("entrypoint"));
        let derive = if is_entrypoint {
            quote! {} // Already derived by #[entrypoint]
        } else {
            quote! {#[cfg_attr(feature = "contract-client-gen", derive(Default))]}
        };

        out.extend(quote! {
            #(#attrs)*
            #[stylus_sdk::stylus_proc::storage]
            #derive
            #vis struct #name #ty_generics #where_clause {
                #fields
            }
        });
    }

    out.into()
}
