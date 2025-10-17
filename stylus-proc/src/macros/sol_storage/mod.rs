// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use proc::{SolidityField, SolidityFields, SolidityStruct, SolidityStructs};
use proc_macro::TokenStream;
use quote::quote;
use syn::token::Colon;
use syn::{
    parse_macro_input, parse_quote, punctuated::Punctuated, PredicateType, Token, TypeParamBound,
    WherePredicate,
};

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

        // Will work also without, but we get more precise error messages when we enforce the bound on the struct
        let predicates = generics
            .type_params()
            .map(|ident| {
                (
                    parse_quote!(#ident),
                    Punctuated::from_iter::<Vec<TypeParamBound>>(vec![parse_quote!(Default)]),
                )
            })
            .map(|(ident, ty_bounds)| {
                WherePredicate::Type(PredicateType {
                    lifetimes: None,
                    bounded_ty: ident,
                    colon_token: Colon::default(),
                    bounds: ty_bounds,
                })
            })
            .collect::<Vec<_>>();
        let where_clause = generics.make_where_clause();
        for p in &predicates {
            where_clause.predicates.push(p.clone());
        }

        let (_, ty_generics, where_clause) = generics.split_for_impl();

        out.extend(quote! {
            #(#attrs)*
            #[stylus_sdk::stylus_proc::storage]
            #vis struct #name #ty_generics #where_clause {
                #fields
            }
        });
    }

    out.into()
}
