// Copyright 2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use cfg_if::cfg_if;
use proc_macro2::TokenStream;
use proc_macro_error::emit_error;
use quote::ToTokens;
use syn::{parse::Nothing, parse_macro_input, parse_quote, Fields};

cfg_if! {
    if #[cfg(feature = "export-abi")] {
        mod export_abi;
        type Extension = export_abi::InnerTypesExtension;
    } else {
        type Extension = ();
    }
}

/// Implementation of the [`#[derive(SolidityError]`][crate::SolidityError] macro.
pub fn derive_solidity_error(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(input as syn::ItemEnum);
    DeriveSolidityError::from(&item).into_token_stream().into()
}

#[derive(Debug)]
struct DeriveSolidityError<E = Extension> {
    name: syn::Ident,
    from_impls: Vec<syn::ItemImpl>,
    match_arms: Vec<syn::Arm>,
    _ext: E,
}

impl DeriveSolidityError {
    fn new(name: syn::Ident) -> Self {
        Self {
            name,
            from_impls: Vec::new(),
            match_arms: Vec::new(),
            _ext: Extension::default(),
        }
    }

    fn add_variant(&mut self, name: &syn::Ident, field: syn::Field) {
        let self_name = &self.name;
        let ty = &field.ty;
        self.from_impls.push(parse_quote! {
            impl From<#ty> for #self_name {
                fn from(value: #ty) -> Self {
                    #self_name::#name(value)
                }
            }
        });
        self.match_arms.push(parse_quote! {
            #self_name::#name(e) => stylus_sdk::call::MethodError::encode(e),
        });
        self._ext.add_variant(field);
    }

    fn vec_u8_from_impl(&self) -> syn::ItemImpl {
        let name = &self.name;
        let match_arms = self.match_arms.iter();
        parse_quote! {
            impl From<#name> for alloc::vec::Vec<u8> {
                fn from(err: #name) -> Self {
                    match err {
                        #(#match_arms)*
                    }
                }
            }
        }
    }
}

impl From<&syn::ItemEnum> for DeriveSolidityError {
    fn from(item: &syn::ItemEnum) -> Self {
        let mut output = DeriveSolidityError::new(item.ident.clone());

        for variant in &item.variants {
            let error = match &variant.fields {
                Fields::Unnamed(e) if variant.fields.len() == 1 => {
                    e.unnamed.first().unwrap().clone()
                }
                _ => {
                    emit_error!(variant.fields, "variant not a 1-tuple");
                    continue;
                }
            };
            output.add_variant(&variant.ident, error);
        }

        output
    }
}

impl ToTokens for DeriveSolidityError {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for from_impl in &self.from_impls {
            from_impl.to_tokens(tokens);
        }
        self.vec_u8_from_impl().to_tokens(tokens);
        Extension::codegen(self).to_tokens(tokens);
    }
}

trait SolidityErrorExtension: Default {
    type Ast: ToTokens;

    fn add_variant(&mut self, field: syn::Field);
    fn codegen(err: &DeriveSolidityError<Self>) -> Self::Ast;
}

impl SolidityErrorExtension for () {
    type Ast = Nothing;

    fn add_variant(&mut self, _field: syn::Field) {}

    fn codegen(_err: &DeriveSolidityError<Self>) -> Self::Ast {
        Nothing
    }
}

#[cfg(test)]
mod tests {}
