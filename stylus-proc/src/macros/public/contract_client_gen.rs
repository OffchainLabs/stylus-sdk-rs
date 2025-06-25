// Copyright 2022-2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use quote::quote;

use crate::{
    consts::STYLUS_CONTRACT_ADDRESS_FIELD, imports::alloy_sol_types::SolType,
    imports::stylus_sdk::abi::AbiType, macros::public::PublicImpl,
};

pub fn generate_client(public_impl: PublicImpl) -> proc_macro2::TokenStream {
    let client_funcs = public_impl
        .funcs
        .iter()
        .map(|func| {
            let func_name = func.name.clone();

            let (context, call) = func.purity.get_context_and_call();

            let inputs = func.inputs.iter().map(|input| {
                let name = input.name.clone();
                let ty = input.ty.clone();
                quote! { #name: #ty }
            });
            let inputs_names = func.inputs.iter().map(|input| {
                input.name.clone()
            });
            let inputs_types = func.inputs.iter().map(|input| {
                let ty = input.ty.clone();
                quote! { #ty }
            });

            let output_type = match &func.output {
                syn::ReturnType::Type(_, ty) => {
                    let ty = ty.clone();
                    quote! { #ty }
                }
                syn::ReturnType::Default => {
                    quote! { () }
                }
            };

            let function_selector = func.function_selector();

            quote! {
                pub fn #func_name(
                    &self,
                    host: &dyn stylus_sdk::stylus_core::host::Host,
                    context: impl #context,
                    #(#inputs,)*
                ) -> Result<<<#output_type as #AbiType>::SolType as #SolType>::RustType, stylus_sdk::stylus_core::calls::errors::Error> {
                    let inputs = <<(#(#inputs_types,)*) as #AbiType>::SolType as #SolType>::abi_encode_params(&(#(#inputs_names,)*));
                    use stylus_sdk::function_selector;
                    let mut calldata = #function_selector;
                    let call_result = #call(host, context, self.#STYLUS_CONTRACT_ADDRESS_FIELD, &calldata)?;
                    Ok(<<#output_type as #AbiType>::SolType as #SolType>::abi_decode(&call_result)?)
                }
            }
        })
        .collect::<proc_macro2::TokenStream>();

    let struct_path = public_impl.self_ty;

    let mut output = quote! {
        impl #struct_path {
            #client_funcs
        }
    };

    // If the impl does not implement a trait, we add a constructor for the contract client
    if public_impl.trait_.is_none() {
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
