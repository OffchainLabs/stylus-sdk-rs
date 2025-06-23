// Copyright 2022-2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use quote::quote;
use convert_case::{Case, Casing};
use sha3::{Digest, Keccak256};

use crate::{
    consts::STYLUS_CONTRACT_ADDRESS_FIELD,
    imports::alloy_sol_types::SolType,
};

fn get_context_and_call(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
) -> Option<(syn::FnArg, proc_macro2::TokenStream)> {
    match inputs.iter().next() {
        Some(syn::FnArg::Receiver(receiver)) => {
            let is_mutable = receiver.mutability.is_some();
            let is_reference = receiver.reference.is_some();

            if is_reference && is_mutable {
                // &mut self
                return Some((syn::parse_quote!(context: stylus_sdk::stylus_core::calls::MutatingCallContext), quote!(stylus_sdk::call::call)));
            } else if is_reference {
                // &self
                return Some((syn::parse_quote!(context: stylus_sdk::stylus_core::calls::StaticCallContext), quote!(stylus_sdk::call::static_call)));
            } else {
                // don't output a method if first argument is not `&self` or `&mut self`
                return None;
            }
        }
        _ => {
            // don't output a method if first argument is not `&self` or `&mut self`
            return None;
        }
    };
}

fn get_new_inputs(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
    context: syn::FnArg,
) -> syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma> {
    let mut new_inputs = syn::punctuated::Punctuated::<syn::FnArg, syn::token::Comma>::new();
    new_inputs.push(syn::parse_quote!(&self));
    new_inputs.push(syn::parse_quote!(host: &dyn stylus_sdk::stylus_core::host::Host));
    new_inputs.push(context);
    new_inputs.extend(inputs.iter().skip(1).cloned());
    new_inputs
}

pub fn generate_client(item_impl: syn::ItemImpl) -> proc_macro2::TokenStream {
    let client_methods = item_impl.items.iter().filter_map(|impl_item| {
        if let syn::ImplItem::Fn(method) = impl_item {
            let sig = &method.sig;
            let method_name = &sig.ident;
            let inputs = &sig.inputs;
            let output = &sig.output;

            let (context, call) = match get_context_and_call(inputs) {
                Some((context, call)) => (context, call),
                None => {
                    // don't output method
                    return None;
                }
            };

            let new_inputs = get_new_inputs(inputs, context);

            let rust_input_types = inputs.iter().skip(1).map(|input| {
                match input {
                    syn::FnArg::Typed(pat_type) => {
                        let ty = &pat_type.ty;
                        quote! { #ty }
                    }
                    _ => panic!("Expected typed argument"),
                }
            });
            let rust_input_names = inputs.iter().skip(1).map(|input| {
                match input {
                    syn::FnArg::Typed(pat_type) => {
                        if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                            pat_ident.ident.clone()
                        } else {
                            panic!("Expected identifier in function argument");
                        }
                    }
                    _ => panic!("Expected typed argument"),
                }
            });

            let rust_output_type = match output {
                syn::ReturnType::Type(_, ty) => {
                    if let syn::Type::Path(syn::TypePath { path, .. }) = &**ty {
                        quote! { #path }
                    } else {
                        panic!("Expected path type in return type");
                    }
                }
                syn::ReturnType::Default => {
                    quote! { () }
                }
            };

            let mut selector = Keccak256::new();
            let method_name_camel = method_name.to_string().to_case(Case::Camel);
            selector.update(method_name_camel);
            selector.update("(");
            for (i, input) in inputs.iter().skip(1).enumerate() {
                if i > 0 {
                    selector.update(",");
                }
                match input {
                    syn::FnArg::Typed(pat_type) => {
                        if let syn::Type::Path(syn::TypePath { path, .. }) = &*pat_type.ty {
                            if let Some(segment) = path.segments.last() {
                                selector.update(segment.ident.to_string());
                            }
                        } else {
                            panic!("Expected path type in function argument");
                        }
                    }
                    _ => panic!("Expected typed argument"),
                }
            }
            selector.update(")");
            let selector_bytes = selector.finalize();
            let selector0 = selector_bytes[0];
            let selector1 = selector_bytes[1];
            let selector2 = selector_bytes[2];
            let selector3 = selector_bytes[3];

            Some(quote! {
                pub fn #method_name(#new_inputs) -> Result<<#rust_output_type as #SolType>::RustType, stylus_sdk::stylus_core::calls::errors::Error> {
                    let args = <(#(#rust_input_types,)*) as #SolType>::abi_encode_params(&(#(#rust_input_names,)*));
                    let mut calldata = vec![#selector0, #selector1, #selector2, #selector3];
                    calldata.extend(args);
                    let call_result = #call(host, context, self.#STYLUS_CONTRACT_ADDRESS_FIELD, &calldata)?;
                    Ok(<#rust_output_type as #SolType>::abi_decode_params(&call_result)?.0)
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
