// Copyright 2022-2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use convert_case::{Case, Casing};
use proc_macro_error::emit_error;
use quote::quote;
use sha3::{Digest, Keccak256};
use syn::spanned::Spanned;

use crate::{consts::STYLUS_CONTRACT_ADDRESS_FIELD, imports::alloy_sol_types::SolType, imports::stylus_sdk::abi::AbiType};

fn get_context_and_call(
    first_input: Option<&syn::FnArg>,
) -> Option<(proc_macro2::TokenStream, proc_macro2::TokenStream)> {
    match first_input {
        Some(syn::FnArg::Receiver(receiver)) => {
            let is_mutable = receiver.mutability.is_some();
            let is_reference = receiver.reference.is_some();

            if is_reference && is_mutable {
                // &mut self
                return Some((
                    quote!(stylus_sdk::stylus_core::calls::MutatingCallContext),
                    quote!(stylus_sdk::call::call),
                ));
            } else if is_reference {
                // &self
                return Some((
                    quote!(stylus_sdk::stylus_core::calls::StaticCallContext),
                    quote!(stylus_sdk::call::static_call),
                ));
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

fn get_inputs_types_and_names<'a>(
    inputs: impl Iterator<Item = &'a syn::FnArg>,
) -> (Vec<proc_macro2::TokenStream>, Vec<syn::Ident>) {
    let mut inputs_types = Vec::new();
    let mut inputs_names = Vec::new();
    for input in inputs {
        match input {
            syn::FnArg::Typed(pat_type) => {
                if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                    let ty = &pat_type.ty;
                    inputs_types.push(quote! { #ty });
                    inputs_names.push(pat_ident.ident.clone());
                } else {
                    emit_error!(input.span(), "Expected identifier in function argument");
                }
            }
            _ => panic!("Expected typed argument"),
        }
    }
    (inputs_types, inputs_names)
}

pub fn generate_client(item_impl: syn::ItemImpl) -> proc_macro2::TokenStream {
    let client_methods = item_impl.items.iter().filter_map(|impl_item| {
        if let syn::ImplItem::Fn(method) = impl_item {
            let method_name = &method.sig.ident;
            let output = &method.sig.output;

            let (context, call) = match get_context_and_call(method.sig.inputs.first()) {
                Some((context, call)) => (context, call),
                None => {
                    // don't output method
                    return None;
                }
            };

            let inputs = method.sig.inputs.iter().skip(1);

            let (inputs_types, inputs_names) = get_inputs_types_and_names(inputs.clone());

            let output_type = match output {
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
            for (i, input) in inputs.clone().enumerate() {
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
                pub fn #method_name(
                    &self,
                    host: &dyn stylus_sdk::stylus_core::host::Host,
                    context: #context, #(#inputs,)*
                ) -> Result<<<#output_type as #AbiType>::SolType as #SolType>::RustType, stylus_sdk::stylus_core::calls::errors::Error> {
                    let inputs = <<(#(#inputs_types,)*) as #AbiType>::SolType as #SolType>::abi_encode_params(&(#(#inputs_names,)*));
                    let mut calldata = vec![#selector0, #selector1, #selector2, #selector3];
                    calldata.extend(inputs);
                    let call_result = #call(host, context, self.#STYLUS_CONTRACT_ADDRESS_FIELD, &calldata)?;
                    Ok(<<#output_type as #AbiType>::SolType as #SolType>::abi_decode_params(&call_result)?)
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
