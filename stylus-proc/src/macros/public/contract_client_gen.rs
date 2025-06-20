// Copyright 2022-2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use quote::quote;

use crate::consts::STYLUS_CONTRACT_ADDRESS_FIELD;

fn get_context_input(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
) -> Option<syn::FnArg> {
    match inputs.iter().next() {
        Some(syn::FnArg::Receiver(receiver)) => {
            let is_mutable = receiver.mutability.is_some();
            let is_reference = receiver.reference.is_some();

            if is_reference && is_mutable {
                // &mut self
                return Some(syn::parse_quote!(context: MutatingCallContext));
            } else if is_reference {
                // &self
                return Some(syn::parse_quote!(context: StaticCallContext));
            } else {
                // don't output a method if first argument is not `&self` or `&mut self`
                return None;
            }
        }
        _ => {
            return None;
        }
    };
}

pub fn generate_client(item_impl: syn::ItemImpl) -> proc_macro2::TokenStream {
    let client_methods = item_impl.items.iter().filter_map(|impl_item| {
        if let syn::ImplItem::Fn(method) = impl_item {
            let sig = &method.sig;
            let method_name = &sig.ident;
            let inputs = &sig.inputs;
            let output = &sig.output;
            let const_token = &sig.constness;

            let mut new_inputs = syn::punctuated::Punctuated::<syn::FnArg, syn::token::Comma>::new();
            new_inputs.push(syn::parse_quote!(&self));

            let context_input = match get_context_input(inputs) {
                Some(input) => input,
                None => {
                    return None;
                }
            };
            new_inputs.push(context_input);

            // adds the rest of the inputs, skipping the first one which should be `&self` or `&mut self`
            new_inputs.extend(inputs.iter().skip(1).cloned());

            let default_return_value = match output {
                syn::ReturnType::Default => quote! { () },
                syn::ReturnType::Type(_, ty) => {
                    match &**ty {
                        syn::Type::Path(syn::TypePath { path, .. }) if path.is_ident("u8") => quote! { 0u8 },
                        syn::Type::Path(syn::TypePath { path, .. }) if path.is_ident("u16") => quote! { 0u16 },
                        syn::Type::Path(syn::TypePath { path, .. }) if path.is_ident("u32") => quote! { 0u32 },
                        syn::Type::Path(syn::TypePath { path, .. }) if path.is_ident("u64") => quote! { 0u64 },
                        syn::Type::Path(syn::TypePath { path, .. }) if path.is_ident("u128") => quote! { 0u128 },
                        syn::Type::Path(syn::TypePath { path, .. }) if path.is_ident("usize") => quote! { 0usize },
                        syn::Type::Path(syn::TypePath { path, .. }) if path.is_ident("i8") => quote! { 0i8 },
                        syn::Type::Path(syn::TypePath { path, .. }) if path.is_ident("i16") => quote! { 0i16 },
                        syn::Type::Path(syn::TypePath { path, .. }) if path.is_ident("i32") => quote! { 0i32 },
                        syn::Type::Path(syn::TypePath { path, .. }) if path.is_ident("i64") => quote! { 0i64 },
                        syn::Type::Path(syn::TypePath { path, .. }) if path.is_ident("i128") => quote! { 0i128 },
                        syn::Type::Path(syn::TypePath { path, .. }) if path.is_ident("isize") => quote! { 0isize },
                        syn::Type::Path(syn::TypePath { path, .. }) if path.is_ident("bool") => quote! { false },
                        syn::Type::Path(syn::TypePath { path, .. }) if path.is_ident("String") => quote! { String::new() },
                        syn::Type::Path(syn::TypePath { path, .. }) if path.is_ident("Address") => quote! { stylus_sdk::alloy_primitives::Address::ZERO },
                        syn::Type::Path(syn::TypePath { path, .. }) if path.is_ident("U256") => quote! { stylus_sdk::alloy_primitives::U256::ZERO },
                        syn::Type::Tuple(syn::TypeTuple { elems, .. }) if elems.is_empty() => quote! { () },
                        _ => {
                            quote! { Default::default() }
                        }
                    }
                }
            };

            Some(quote! {
                #const_token pub fn #method_name (#new_inputs) #output {
                    println!("(Simulated Call) Executing method: {}", stringify!(#method_name));
                    #default_return_value
                }
            })
        } else {
            return None;
        }
    }).collect::<proc_macro2::TokenStream>();

    let struct_path = item_impl.self_ty;

    let mut output = quote! {
        impl #struct_path {
            #client_methods
        }
    };

    // If the impl does not implement a trait, we add a constructor for the contract client
    if item_impl.trait_.is_none() {
        output.extend(quote! {
            impl #struct_path {
                pub fn new(address: stylus_sdk::alloy_primitives::Address) -> Self {
                    Self {
                        #STYLUS_CONTRACT_ADDRESS_FIELD: address,
                    }
                }
            }
        });
    }

    output.into()
}
