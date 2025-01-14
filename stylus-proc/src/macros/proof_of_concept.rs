// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use proc_macro2::TokenStream;
use proc_macro_error::emit_error;
use quote::ToTokens;
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, GenericParam, ItemStruct, Path, Type,
    TypePath, WhereClause,
};

use crate::consts::STYLUS_HOST_FIELD;

/// Implementation of the [`#[proof_of_concept]`][crate::storage] macro.
pub fn proof_of_concept(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    if !attr.is_empty() {
        emit_error!(
            TokenStream::from(attr).span(),
            "this macro is not configurable"
        );
    }

    let item = parse_macro_input!(input as ItemStruct);
    HostInjectedStruct::new(item)
        .item
        .into_token_stream()
        .into()
}

pub struct HostInjectedStruct {
    pub item: ItemStruct,
}

impl HostInjectedStruct {
    pub fn new(mut item: ItemStruct) -> Self {
        let host_param = ensure_host_generic_param(&mut item.generics);

        // Then transform all fields that need the host parameter generic.
        transform_fields(&mut item, &host_param);

        // Finally add the host field to the item struct.
        add_host_field(&mut item, &host_param);

        Self { item }
    }
}

fn ensure_host_generic_param(generics: &mut syn::Generics) -> syn::Ident {
    // Check if any generic parameter has the Host bound
    if let Some(param) = find_host_param(generics) {
        return param;
    }

    // If not found, add a new one with a unique name
    let host_ident = generate_unique_host_param(generics);
    let host_param: GenericParam = parse_quote!(#host_ident: stylus_sdk::host::Host);
    generics.params.push(host_param);
    host_ident
}

fn find_host_param(generics: &syn::Generics) -> Option<syn::Ident> {
    // Check generic parameters
    if let Some(ident) = find_host_in_params(&generics.params) {
        return Some(ident);
    }

    // Check where clause
    if let Some(where_clause) = &generics.where_clause {
        return find_host_in_where_clause(where_clause);
    }

    None
}

fn find_host_in_params(
    params: &syn::punctuated::Punctuated<GenericParam, syn::Token![,]>,
) -> Option<syn::Ident> {
    params
        .iter()
        .filter_map(|param| match param {
            GenericParam::Type(type_param) if has_host_bound(&type_param.bounds) => {
                Some(type_param.ident.clone())
            }
            _ => None,
        })
        .next()
}

fn find_host_in_where_clause(where_clause: &WhereClause) -> Option<syn::Ident> {
    where_clause
        .predicates
        .iter()
        .filter_map(|pred| match pred {
            syn::WherePredicate::Type(pred_type) => match &pred_type.bounded_ty {
                Type::Path(TypePath { path, .. })
                    if path.segments.len() == 1 && has_host_bound(&pred_type.bounds) =>
                {
                    Some(path.segments[0].ident.clone())
                }
                _ => None,
            },
            _ => None,
        })
        .next()
}

fn has_host_bound(
    bounds: &syn::punctuated::Punctuated<syn::TypeParamBound, syn::Token![+]>,
) -> bool {
    bounds.iter().any(|bound| {
        if let syn::TypeParamBound::Trait(trait_bound) = bound {
            is_host_path(&trait_bound.path)
        } else {
            false
        }
    })
}

fn is_host_path(path: &Path) -> bool {
    let segments = &path.segments;
    segments.len() == 3
        && segments[0].ident == "stylus_sdk"
        && segments[1].ident == "host"
        && segments[2].ident == "Host"
}

fn generate_unique_host_param(generics: &syn::Generics) -> syn::Ident {
    let mut counter = 0;
    loop {
        let name = if counter == 0 {
            "H".to_string()
        } else {
            format!("H{}", counter)
        };

        let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());

        if !generics.params.iter().any(|param| {
            if let GenericParam::Type(type_param) = param {
                type_param.ident == ident
            } else {
                false
            }
        }) {
            return ident;
        }
        counter += 1;
    }
}

fn transform_fields(item: &mut ItemStruct, host_param: &syn::Ident) {
    match &mut item.fields {
        syn::Fields::Named(fields) => {
            for field in &mut fields.named {
                transform_type(&mut field.ty, host_param);
            }
        }
        syn::Fields::Unnamed(_) => {
            emit_error!(
                item.fields.span(),
                "Tuple structs are not supported by #[storage]"
            );
        }
        syn::Fields::Unit => {}
    }
}

fn transform_type(ty: &mut Type, host_param: &syn::Ident) {
    if let Type::Path(type_path) = ty {
        if is_storage_type(&type_path.path) {
            // Get the last path segment
            if let Some(last_segment) = type_path.path.segments.last_mut() {
                match &mut last_segment.arguments {
                    syn::PathArguments::None => {
                        // Construct angle bracketed args with host param
                        let mut args = syn::punctuated::Punctuated::new();
                        args.push(syn::GenericArgument::Type(parse_quote!(#host_param)));
                        last_segment.arguments = syn::PathArguments::AngleBracketed(
                            syn::AngleBracketedGenericArguments {
                                colon2_token: None,
                                lt_token: syn::token::Lt::default(),
                                args,
                                gt_token: syn::token::Gt::default(),
                            },
                        );
                    }
                    syn::PathArguments::AngleBracketed(args) => {
                        // Check if any of the existing arguments is a Host type
                        let has_host = args.args.iter().any(|arg| {
                            if let syn::GenericArgument::Type(Type::Path(p)) = arg {
                                is_host_path(&p.path)
                            } else {
                                false
                            }
                        });

                        if !has_host {
                            // Add host parameter to existing arguments
                            args.args.push(parse_quote!(#host_param));
                        }
                    }
                    syn::PathArguments::Parenthesized(_) => {
                        // Shouldn't happen with normal Rust types
                        emit_error!(
                            last_segment.span(),
                            "Parenthesized arguments are not supported"
                        );
                    }
                }
            }
        }

        // Recursively transform generic arguments
        if let Some(args) = get_type_arguments_mut(ty) {
            for arg in args {
                transform_type(arg, host_param);
            }
        }
    }
}

fn is_storage_type(path: &Path) -> bool {
    if path.segments.is_empty() {
        return false;
    }

    let first_segment = &path.segments[0].ident;
    first_segment.to_string().starts_with("Storage")
}

fn get_type_arguments_mut(ty: &mut Type) -> Option<Vec<&mut Type>> {
    let Type::Path(TypePath { path, .. }) = ty else {
        return None;
    };

    let last_segment = path.segments.last_mut()?;

    let syn::PathArguments::AngleBracketed(args) = &mut last_segment.arguments else {
        return None;
    };

    let type_args = args
        .args
        .iter_mut()
        .filter_map(|arg| match arg {
            syn::GenericArgument::Type(ty) => Some(ty),
            _ => None,
        })
        .collect();

    Some(type_args)
}

fn add_host_field(item: &mut ItemStruct, host_param: &syn::Ident) {
    match &mut item.fields {
        syn::Fields::Named(fields) => {
            fields.named.push(parse_quote! {
                #STYLUS_HOST_FIELD: *const #host_param
            });
        }
        syn::Fields::Unit => {
            let mut named = syn::punctuated::Punctuated::new();
            named.push(parse_quote! {
                #STYLUS_HOST_FIELD: *const #host_param
            });
            item.fields = syn::Fields::Named(syn::FieldsNamed {
                brace_token: syn::token::Brace::default(),
                named,
            });
        }
        syn::Fields::Unnamed(_) => {
            emit_error!(
                item.fields.span(),
                "Tuple structs are not supported by #[storage]"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::HostInjectedStruct;
    use crate::utils::testing::assert_ast_eq;
    use syn::{parse_quote, ItemStruct};

    #[test]
    fn test_proof_of_concept() {
        let cases: Vec<(ItemStruct, ItemStruct)> = vec![
            // Unit struct works.
            (
                parse_quote! {
                    #[proof_of_concept]
                    pub struct Counter;
                },
                parse_quote! {
                    #[proof_of_concept]
                    pub struct Counter<H: stylus_sdk::host::Host> {
                        __stylus_host: *const H,
                    }
                },
            ),
            (
                parse_quote! {
                    #[proof_of_concept]
                    pub struct Counter<H: stylus_sdk::host::Host>;
                },
                parse_quote! {
                    #[proof_of_concept]
                    pub struct Counter<H: stylus_sdk::host::Host> {
                        __stylus_host: *const H,
                    }
                },
            ),
            // Using a different generic parameter name works, and preserves it across fields.
            (
                parse_quote! {
                    #[proof_of_concept]
                    pub struct Counter<M: stylus_sdk::host::Host>;
                },
                parse_quote! {
                    #[proof_of_concept]
                    pub struct Counter<M: stylus_sdk::host::Host> {
                        __stylus_host: *const M,
                    }
                },
            ),
            // Using a struct that already uses H for another generic param works by adding
            // a new identifier that does not conflict.
            (
                parse_quote! {
                    #[proof_of_concept]
                    pub struct Counter<H: std::fmt::Display>;
                },
                parse_quote! {
                    #[proof_of_concept]
                    pub struct Counter<H: std::fmt::Display, H1: stylus_sdk::host::Host> {
                        __stylus_host: *const H1,
                    }
                },
            ),
            // Basic structs with storage fields work.
            (
                parse_quote! {
                    #[proof_of_concept]
                    pub struct Counter {
                        number: StorageU256,
                    }
                },
                parse_quote! {
                    #[proof_of_concept]
                    pub struct Counter<H: stylus_sdk::host::Host> {
                        number: StorageU256<H>,
                        __stylus_host: *const H,
                    }
                },
            ),
            (
                parse_quote! {
                    #[proof_of_concept]
                    pub struct Counter {
                        number: StorageBool,
                    }
                },
                parse_quote! {
                    #[proof_of_concept]
                    pub struct Counter<H: stylus_sdk::host::Host> {
                        number: StorageBool<H>,
                        __stylus_host: *const H,
                    }
                },
            ),
            // Structs with storage fields and already used parameter works.
            (
                parse_quote! {
                    #[proof_of_concept]
                    pub struct Counter<M: stylus_sdk::host::Host> {
                        number: StorageBool,
                    }
                },
                parse_quote! {
                    #[proof_of_concept]
                    pub struct Counter<M: stylus_sdk::host::Host> {
                        number: StorageBool<M>,
                        __stylus_host: *const M,
                    }
                },
            ),
            // Structs with storage fields that have other generics still injects host.
            (
                parse_quote! {
                    #[proof_of_concept]
                    pub struct Counter<M: std::fmt::Display> {
                        number: StorageUint<M>,
                    }
                },
                parse_quote! {
                    #[proof_of_concept]
                    pub struct Counter<M: std::fmt::Display, H: stylus_sdk::host::Host> {
                        number: StorageUint<M, H>,
                        __stylus_host: *const H,
                    }
                },
            ),
            // Structs with storage fields that have other generics, but that already use the H identifier
            // still injects host as a separate identifier that does not conflict.
            (
                parse_quote! {
                    #[proof_of_concept]
                    pub struct Counter<H: std::fmt::Display> {
                        number: StorageUint<H>,
                    }
                },
                parse_quote! {
                    #[proof_of_concept]
                    pub struct Counter<H: std::fmt::Display, H1: stylus_sdk::host::Host> {
                        number: StorageUint<H, H2>,
                        __stylus_host: *const H1,
                    }
                },
            ),
        ];
        for case in cases {
            let got = HostInjectedStruct::new(case.0);
            assert_ast_eq(got.item, case.1);
        }
    }
}
