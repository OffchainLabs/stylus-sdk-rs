// Copyright 2022-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md
use proc_macro2::{Span, TokenStream};
use proc_macro_error::emit_error;
use quote::{quote, ToTokens};
use syn::{
    parse::Nothing, parse_quote, parse_quote_spanned, punctuated::Punctuated, spanned::Spanned,
    Token,
};

use crate::{
    imports::{
        alloy_sol_types::SolType,
        stylus_sdk::abi::{AbiType, Router},
    },
    types::Purity,
};

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
        let function_iter = self
            .funcs
            .iter()
            .filter(|&func| matches!(func.kind, FnKind::Function));
        let selector_consts = function_iter.clone().map(PublicFn::selector_const);
        let selector_arms = function_iter
            .map(PublicFn::selector_arm)
            .collect::<Vec<_>>();
        let inheritance_routes = self.inheritance_routes();

        let call_fallback = self.call_fallback();
        let inheritance_fallback = self.inheritance_fallback();

        let (fallback, fallback_purity) = call_fallback.unwrap_or_else(|| {
            // If there is no fallback function specified, we rely on any inherited fallback.
            (
                parse_quote!({
                    #(#inheritance_fallback)*
                    None
                }),
                Purity::Payable, // Let the inherited fallback deal with purity.
            )
        });

        let fallback_deny: Option<syn::ExprIf> = match fallback_purity {
            Purity::Payable => None,
            _ => Some(parse_quote! {
                if let Err(err) = stylus_sdk::abi::internal::deny_value("fallback") {
                    return Some(Err(err));
                }
            }),
        };

        let call_receive = self.call_receive();
        let inheritance_receive = self.inheritance_receive();
        let receive = call_receive.unwrap_or_else(|| {
            parse_quote!({
                #(#inheritance_receive)*
                None
            })
        });

        parse_quote! {
            impl<S, #generic_params> #Router<S> for #self_ty
            where
                S: stylus_core::context::TopLevelStorage + core::borrow::BorrowMut<Self>,
                #(
                    S: core::borrow::BorrowMut<#inheritance>,
                )*
                #where_clause
            {
                type Storage = Self;

                #[inline(always)]
                #[deny(unreachable_patterns)]
                fn route(storage: &mut S, selector: u32, input: &[u8]) -> Option<stylus_sdk::ArbResult> {
                    use stylus_sdk::function_selector;
                    use stylus_sdk::abi::{internal, internal::EncodableReturnType};
                    use alloc::vec;

                    #(#selector_consts)*
                    match selector {
                        #(#selector_arms)*
                        _ => {
                            #(#inheritance_routes)*
                            None
                        }
                    }
                }

                #[inline(always)]
                fn fallback(storage: &mut S, input: &[u8]) -> Option<stylus_sdk::ArbResult> {
                    #fallback_deny
                    #fallback
                }

                #[inline(always)]
                fn receive(storage: &mut S) -> Option<Result<(), Vec<u8>>> {
                    #receive
                }
            }
        }
    }

    fn inheritance_routes(&self) -> impl Iterator<Item = syn::ExprIf> + '_ {
        self.inheritance.iter().map(|ty| {
            parse_quote! {
                if let Some(result) = <#ty as #Router<S>>::route(storage, selector, input) {
                    return Some(result);
                }
            }
        })
    }

    fn call_fallback(&self) -> Option<(syn::Stmt, Purity)> {
        let mut fallback_purity = Purity::View;
        let fallbacks: Vec<syn::Stmt> = self
            .funcs
            .iter()
            .filter(|&func| {
                if matches!(func.kind, FnKind::FallbackWithArgs)
                    || matches!(func.kind, FnKind::FallbackNoArgs)
                {
                    fallback_purity = func.purity;
                    return true;
                }
                false
            })
            .map(PublicFn::call_fallback)
            .collect();
        if fallbacks.is_empty() {
            return None;
        }
        if fallbacks.len() > 1 {
            emit_error!(
                "multiple fallbacks",
                "contract can only have one #[fallback] method defined"
            );
        }
        fallbacks
            .first()
            .cloned()
            .map(|func| (func, fallback_purity))
    }

    fn inheritance_fallback(&self) -> impl Iterator<Item = syn::ExprIf> + '_ {
        self.inheritance.iter().map(|ty| {
            parse_quote! {
                if let Some(res) = <#ty as #Router<S>>::fallback(storage, input) {
                    return Some(res);
                }
            }
        })
    }

    fn call_receive(&self) -> Option<syn::Stmt> {
        let receives: Vec<syn::Stmt> = self
            .funcs
            .iter()
            .filter(|&func| matches!(func.kind, FnKind::Receive))
            .map(PublicFn::call_receive)
            .collect();
        if receives.is_empty() {
            return None;
        }
        if receives.len() > 1 {
            emit_error!(
                "multiple receives",
                "contract can only have one #[receive] method defined"
            );
        }
        receives.first().cloned()
    }

    fn inheritance_receive(&self) -> impl Iterator<Item = syn::ExprIf> + '_ {
        self.inheritance.iter().map(|ty| {
            parse_quote! {
                if let Some(res) = <#ty as #Router<S>>::receive(storage) {
                    return Some(res);
                }
            }
        })
    }
}

#[derive(Debug)]
pub enum FnKind {
    Function,
    FallbackWithArgs,
    FallbackNoArgs,
    Receive,
}

pub struct PublicFn<E: FnExtension> {
    pub name: syn::Ident,
    pub sol_name: syn_solidity::SolIdent,
    pub purity: Purity,
    pub inferred_purity: Purity,
    pub kind: FnKind,

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

    pub fn selector_const(&self) -> Option<syn::ItemConst> {
        let name = self.selector_name();
        let value = self.selector_value();
        Some(parse_quote! {
            #[allow(non_upper_case_globals)]
            const #name: u32 = #value;
        })
    }

    fn selector_arm(&self) -> Option<syn::Arm> {
        if !matches!(self.kind, FnKind::Function) {
            return None;
        }

        let name = &self.name;
        let constant = self.selector_name();
        let deny_value = self.deny_value();
        let decode_inputs = self.decode_inputs();
        let storage_arg = self.storage_arg();
        let expand_args = self.expand_args();
        let encode_output = self.encode_output();
        Some(parse_quote! {
            #[allow(non_upper_case_globals)]
            #constant => {
                #deny_value
                let args = match <#decode_inputs as #SolType>::abi_decode_params(input, true) {
                    Ok(args) => args,
                    Err(err) => {
                        internal::failed_to_decode_arguments(err);
                        return Some(Err(Vec::new()));
                    }
                };
                let result = Self::#name(#storage_arg #(#expand_args, )* );
                Some(#encode_output)
            }
        })
    }

    fn decode_inputs(&self) -> syn::Type {
        let arg_types = self.arg_types();
        parse_quote_spanned! {
            self.input_span => <(#( #arg_types, )*) as #AbiType>::SolType
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

    fn call_fallback(&self) -> syn::Stmt {
        let name = &self.name;
        let storage_arg = self.storage_arg();
        if matches!(self.kind, FnKind::FallbackNoArgs) {
            return parse_quote! {
                return Some({
                    if let Err(err) = Self::#name(#storage_arg) {
                       Err(err)
                    } else {
                        Ok(Vec::new())
                    }
                });
            };
        }
        parse_quote! {
            return Some(Self::#name(#storage_arg input));
        }
    }

    fn call_receive(&self) -> syn::Stmt {
        let name = &self.name;
        let storage_arg = self.storage_arg();
        parse_quote! {
            return Some(Self::#name(#storage_arg));
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
