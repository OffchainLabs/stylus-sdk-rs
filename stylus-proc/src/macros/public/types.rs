// Copyright 2022-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::Nothing, parse_quote, parse_quote_spanned, punctuated::Punctuated, spanned::Spanned,
    Token,
};

use crate::types::Purity;

use super::Extension;

pub struct PublicImpl<E: InterfaceExtension = Extension> {
    pub self_ty: syn::Type,
    pub generic_params: Punctuated<syn::GenericParam, Token![,]>,
    pub where_clause: Punctuated<syn::WherePredicate, Token![,]>,
    pub inheritance: Vec<syn::Type>,
    pub funcs: Vec<PublicFn<E::FnExt>>,
    #[allow(dead_code)]
    pub extension: E,
}

impl PublicImpl {
    pub fn impl_router(&self) -> syn::ItemImpl {
        let Self {
            self_ty,
            generic_params,
            where_clause,
            inheritance,
            ..
        } = self;
        let selector_consts = self.funcs.iter().map(PublicFn::selector_const);
        let selector_arms = self.funcs.iter().map(PublicFn::selector_arm);
        let inheritance_routes = self.inheritance_routes();
        parse_quote! {
            impl<S, #generic_params> stylus_sdk::abi::Router<S> for #self_ty
            where
                S: stylus_sdk::storage::TopLevelStorage + core::borrow::BorrowMut<Self>,
                #(
                    S: core::borrow::BorrowMut<#inheritance>,
                )*
                #where_clause
            {
                type Storage = Self;

                #[inline(always)]
                #[deny(unreachable_patterns)]
                fn route(storage: &mut S, selector: u32, input: &[u8]) -> Option<stylus_sdk::ArbResult> {
                    use stylus_sdk::{function_selector, alloy_sol_types::SolType};
                    use stylus_sdk::abi::{internal, internal::EncodableReturnType, AbiType, Router};
                    use alloc::vec;

                    #[cfg(feature = "export-abi")]
                    use stylus_sdk::abi::export;

                    #(#selector_consts)*
                    match selector {
                        #(#selector_arms)*
                        _ => {
                            #(#inheritance_routes)*
                            None
                        }
                    }
                }
            }
        }
    }

    fn inheritance_routes(&self) -> impl Iterator<Item = syn::ExprIf> + '_ {
        self.inheritance.iter().map(|ty| {
            parse_quote! {
                if let Some(result) = <#ty as Router<S>>::route(storage, selector, input) {
                    return Some(result);
                }
            }
        })
    }
}

pub struct PublicFn<E: FnExtension> {
    pub name: syn::Ident,
    pub sol_name: syn_solidity::SolIdent,
    pub purity: Purity,
    pub inferred_purity: Purity,

    pub has_self: bool,
    pub inputs: Vec<PublicFnArg<E::FnArgExt>>,
    pub input_span: Span,
    pub output_span: Span,

    #[allow(dead_code)]
    pub extension: E,
}

impl<E: FnExtension> PublicFn<E> {
    pub fn selector_name(&self) -> syn::Ident {
        syn::Ident::new(&format!("__SELECTOR_{}", self.name), self.name.span())
    }

    fn selector_value(&self) -> syn::Expr {
        let sol_name = syn::LitStr::new(&self.sol_name.as_string(), self.sol_name.span());
        let arg_types = self.arg_types();
        parse_quote! {
            u32::from_be_bytes(function_selector!(#sol_name #(, #arg_types )*))
        }
    }

    pub fn selector_const(&self) -> syn::ItemConst {
        let name = self.selector_name();
        let value = self.selector_value();
        parse_quote! {
            #[allow(non_upper_case_globals)]
            const #name: u32 = #value;
        }
    }

    fn selector_arm(&self) -> syn::Arm {
        let name = &self.name;
        let constant = self.selector_name();
        let deny_value = self.deny_value();
        let decode_inputs = self.decode_inputs();
        let storage_arg = self.storage_arg();
        let expand_args = self.expand_args();
        let encode_output = self.encode_output();
        parse_quote! {
            #[allow(non_upper_case_globals)]
            #constant => {
                #deny_value
                let args = match <#decode_inputs as SolType>::abi_decode_params(input, true) {
                    Ok(args) => args,
                    Err(err) => {
                        internal::failed_to_decode_arguments(err);
                        return Some(Err(Vec::new()));
                    }
                };
                let result = Self::#name(#storage_arg #(#expand_args, )* );
                Some(#encode_output)
            }
        }
    }

    fn decode_inputs(&self) -> syn::Type {
        let arg_types = self.arg_types();
        parse_quote_spanned! {
            self.input_span => <(#( #arg_types, )*) as AbiType>::SolType
        }
    }

    fn arg_types(&self) -> impl Iterator<Item = &syn::Type> {
        self.inputs.iter().map(|arg| &arg.ty)
    }

    fn storage_arg(&self) -> TokenStream {
        if self.inferred_purity == Purity::Pure {
            quote!()
        } else if self.has_self {
            quote! { core::borrow::BorrowMut::borrow_mut(storage), }
        } else {
            quote! { storage, }
        }
    }

    fn expand_args(&self) -> impl Iterator<Item = syn::Expr> + '_ {
        self.arg_types().enumerate().map(|(index, ty)| {
            let index = syn::Index {
                index: index as u32,
                span: ty.span(),
            };
            parse_quote! { args.#index }
        })
    }

    fn encode_output(&self) -> syn::Expr {
        parse_quote_spanned! {
            self.output_span => EncodableReturnType::encode(result)
        }
    }

    fn deny_value(&self) -> Option<syn::ExprIf> {
        if self.purity == Purity::Payable {
            None
        } else {
            let name = self.name.to_string();
            Some(parse_quote! {
                if let Err(err) = internal::deny_value(#name) {
                    return Some(Err(err));
                }
            })
        }
    }
}

pub struct PublicFnArg<E: FnArgExtension> {
    pub ty: syn::Type,
    #[allow(dead_code)]
    pub extension: E,
}

pub trait InterfaceExtension: Sized {
    type FnExt: FnExtension;
    type Ast: ToTokens;

    fn build(node: &syn::ItemImpl) -> Self;
    fn codegen(iface: &PublicImpl<Self>) -> Self::Ast;
}

pub trait FnExtension {
    type FnArgExt: FnArgExtension;

    fn build(node: &syn::ImplItemFn) -> Self;
}

pub trait FnArgExtension {
    fn build(node: &syn::FnArg) -> Self;
}

impl InterfaceExtension for () {
    type FnExt = ();
    type Ast = Nothing;

    fn build(_node: &syn::ItemImpl) -> Self {}

    fn codegen(_iface: &PublicImpl<Self>) -> Self::Ast {
        Nothing
    }
}

impl FnExtension for () {
    type FnArgExt = ();

    fn build(_node: &syn::ImplItemFn) -> Self {}
}

impl FnArgExtension for () {
    fn build(_node: &syn::FnArg) -> Self {}
}
