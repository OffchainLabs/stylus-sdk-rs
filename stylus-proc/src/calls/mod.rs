// Copyright 2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use crate::types::solidity_type_info;
use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use sha3::{Digest, Keccak256};
use std::borrow::Cow;
use syn_solidity::{FunctionAttribute, Item, Mutability, SolIdent, Spanned, Visibility};

pub fn sol_interface(input: TokenStream) -> TokenStream {
    let input = match syn_solidity::parse(input) {
        Ok(f) => f,
        Err(err) => return err.to_compile_error().into(),
    };

    use crate::types::Purity::*;
    let alloy_address = quote!(stylus_sdk::alloy_primitives::Address);
    let sol_address = quote!(stylus_sdk::alloy_sol_types::sol_data::Address);
    let sol_type = quote!(stylus_sdk::alloy_sol_types::SolType);
    let sol_value = quote!(stylus_sdk::alloy_sol_types::SolValue);
    let sol_type_value = quote!(stylus_sdk::alloy_sol_types::private::SolTypeValue);

    let mut output = quote!();

    for item in input.items {
        let mut method_impls = quote!();

        let Item::Contract(contract) = item else {
            error!(item.span(), "not an interface")
        };
        if !contract.is_interface() {
            error!(contract.kind.span(), "not an interface");
        }

        let name = &contract.name;

        for item in contract.body {
            let Item::Function(func) = item else {
                continue; // ignore non-functions
            };
            // uncomment when Alloy exposes this enum
            //     if let FunctionKind::Function(_) = func.kind {
            //         continue;
            //     }
            let Some(name) = &func.name else {
                continue;
            };

            // determine the purity
            let mut purity = None;
            for attr in &func.attributes.0 {
                match attr {
                    FunctionAttribute::Mutability(mutability) => {
                        if purity.is_some() {
                            error!(attr.span(), "more than one purity attribute specified");
                        }
                        purity = Some(match mutability {
                            Mutability::Constant(_) | Mutability::Pure(_) => Pure,
                            Mutability::View(_) => View,
                            Mutability::Payable(_) => Payable,
                        });
                    }
                    FunctionAttribute::Visibility(vis) => {
                        if let Visibility::Internal(_) | Visibility::Private(_) = vis {
                            error!(vis.span(), "internal method in interface");
                        }
                    }
                    _ => error!(attr.span(), "unsupported function attribute"),
                }
            }
            let purity = purity.unwrap_or(Write);

            // determine which context and kind of call to use
            let (context, call) = match purity {
                Pure | View => (
                    quote! { impl stylus_sdk::call::StaticCallContext },
                    quote! { stylus_sdk::call::static_call },
                ),
                Write => (
                    quote! { impl stylus_sdk::call::NonPayableCallContext },
                    quote! { stylus_sdk::call::call },
                ),
                Payable => (
                    quote! { impl stylus_sdk::call::MutatingCallContext },
                    quote! { stylus_sdk::call::call },
                ),
            };

            macro_rules! parse {
                ($data:expr) => {
                    match syn::parse_str(&$data) {
                        Ok(ty) => ty,
                        Err(err) => return err.to_compile_error().into(),
                    }
                };
            }

            // get the return type
            let return_type = match func.return_type() {
                Some(ty) => solidity_type_info(&ty).0,
                None => Cow::from("()"),
            };
            let return_type: syn::Type = parse!(&return_type);

            let mut selector = Keccak256::new();
            selector.update(name.to_string());
            selector.update("(");
            let mut sol_args = vec![];
            let mut rust_args = vec![];
            let mut rust_arg_names = vec![];
            for (i, arg) in func.parameters.iter().enumerate() {
                let (sol_path, abi) = solidity_type_info(&arg.ty);
                if i > 0 {
                    selector.update(",");
                }
                selector.update(&*abi);

                let ty: syn::Type = parse!(&sol_path);
                let name = arg
                    .name
                    .as_ref()
                    .map(Cow::Borrowed)
                    .unwrap_or_else(|| Cow::Owned(SolIdent::new(&format!("argument_{}", i))));

                rust_args.push(quote! {
                    #name: <#ty as #sol_type>::RustType
                });
                sol_args.push(ty);
                rust_arg_names.push(name);
            }
            selector.update(")");

            let selector = selector.finalize();
            let selector0 = selector[0];
            let selector1 = selector[1];
            let selector2 = selector[2];
            let selector3 = selector[3];

            let rust_name = Ident::new(&name.to_string().to_case(Case::Snake), name.span());

            method_impls.extend(quote! {
                pub fn #rust_name(&self, context: #context #(, #rust_args)*) ->
                    Result<<#return_type as #sol_type>::RustType, stylus_sdk::call::Error>
                {
                    use alloc::vec;
                    let args = <(#(#sol_args,)*) as #sol_type>::abi_encode(&(#(#rust_arg_names,)*));
                    let mut calldata = vec![#selector0, #selector1, #selector2, #selector3];
                    calldata.extend(args);
                    let returned = #call(context, self.address, &calldata)?;
                    Ok(<(#return_type,) as #sol_type>::abi_decode_params(&returned, true)?.0)
                }
            });
        }

        output.extend(quote! {
            pub struct #name {
                pub address: #alloy_address,
            }

            impl #name {
                pub fn new(address: #alloy_address) -> Self {
                    Self { address }
                }

                #method_impls
            }

            impl core::ops::Deref for #name {
                type Target = #alloy_address;

                fn deref(&self) -> &Self::Target {
                    &self.address
                }
            }

            impl #sol_value for #name {
                type SolType = #name;
            }

            impl #sol_type_value<Self> for #name {
                #[inline]
                fn stv_to_tokens(&self) -> <Self as alloy_sol_types::SolType>::Token<'_> {
                    <#sol_address as #sol_type>::tokenize(&self.address)
                }

                #[inline]
                fn stv_abi_encoded_size(&self) -> usize {
                    <#sol_address as #sol_type>::abi_encoded_size(&self.address)
                }

                #[inline]
                fn stv_abi_packed_encoded_size(&self) -> usize {
                    <#sol_address as #sol_type>::abi_packed_encoded_size(&self.address)
                }

                #[inline]
                fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
                    <#sol_address as #sol_type>::eip712_data_word(&self.address)
                }

                #[inline]
                fn stv_abi_encode_packed_to(&self, out: &mut alloy_sol_types::private::Vec<u8>) {
                    <#sol_address as #sol_type>::abi_encode_packed_to(&self.address, out)
                }
            }

            impl #sol_type for #name {
                type RustType = #name;

                type Token<'a> = <#sol_address as #sol_type>::Token<'a>;

                const SOL_NAME: &'static str = <#sol_address as #sol_type>::SOL_NAME;

                const ENCODED_SIZE: Option<usize> = <#sol_address as #sol_type>::ENCODED_SIZE;
                const PACKED_ENCODED_SIZE: Option<usize> = <#sol_address as #sol_type>::PACKED_ENCODED_SIZE;

                fn valid_token(token: &Self::Token<'_>) -> bool {
                    <#sol_address as #sol_type>::valid_token(token)
                }

                fn detokenize(token: Self::Token<'_>) -> Self::RustType {
                    #name::new(#sol_address::detokenize(token))
                }
            }

            impl stylus_sdk::abi::AbiType for #name {
                type SolType = #name;

                const ABI: stylus_sdk::abi::ConstString = <#alloy_address as stylus_sdk::abi::AbiType>::ABI;
            }
        });
    }
    output.into()
}
