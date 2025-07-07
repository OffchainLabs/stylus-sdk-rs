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
    consts::STYLUS_CONTRACT_ADDRESS_FIELD,
    imports::{
        alloy_sol_types::SolType,
        stylus_sdk::abi::{AbiType, Router},
    },
    types::Purity,
};

use super::Extension;

/// Generate the code to call the special function (fallback, receive, or constructor) from the
/// public impl block. Emits an error if there are multiple implementations.
macro_rules! call_special {
    ($self:expr, $kind:pat, $kind_name:literal, $func:expr) => {{
        let specials: Vec<syn::Stmt> = $self
            .funcs
            .iter()
            .filter(|&func| matches!(func.kind, $kind))
            .map($func)
            .collect();
        if specials.is_empty() {
            None
        } else {
            if specials.len() > 1 {
                emit_error!(
                    concat!("multiple ", $kind_name),
                    concat!(
                        "contract can only have one #[",
                        $kind_name,
                        "] method defined"
                    )
                );
            }
            specials.first().cloned()
        }
    }};
}

pub struct PublicImpl<E: InterfaceExtension = Extension> {
    pub self_ty: syn::Type,
    pub generic_params: Punctuated<syn::GenericParam, Token![,]>,
    pub where_clause: Punctuated<syn::WherePredicate, Token![,]>,
    pub trait_: Option<syn::Path>,
    pub implements: Vec<syn::Type>,
    pub funcs: Vec<PublicFn<E::FnExt>>,
    pub associated_types: Vec<(syn::Ident, syn::Type)>,
    #[allow(dead_code)]
    pub extension: E,
}

fn get_default_output(ty: &syn::Type) -> (TokenStream, TokenStream) {
    (
        quote! {
            Result<<<#ty as #AbiType>::SolType as #SolType>::RustType, stylus_sdk::stylus_core::calls::errors::Error>
        },
        quote! {
            Ok(<<#ty as #AbiType>::SolType as #SolType>::abi_decode(&call_result)?)
        },
    )
}

fn get_output_type_and_decoding(output: &syn::ReturnType) -> (TokenStream, TokenStream) {
    match output {
        syn::ReturnType::Default => (
            quote! { Result<(), stylus_sdk::stylus_core::calls::errors::Error> },
            quote! { Ok(()) },
        ),
        syn::ReturnType::Type(_, ty) => {
            // Check if it's a path type (like Result<T, E> or ArbResult)
            let type_path = match ty.as_ref() {
                syn::Type::Path(type_path) => type_path,
                _ => return get_default_output(ty),
            };

            // Check if the path is "Result" or "ArbResult"
            let segment = match type_path.path.segments.last() {
                Some(segment) => segment,
                None => {
                    emit_error!(ty.span(), "Expected a type path with segments, found none");
                    return get_default_output(ty);
                }
            };
            match segment.ident.to_string().as_str() {
                "ArbResult" => (
                    quote! {
                        stylus_sdk::ArbResult
                    },
                    quote! {
                        use stylus_sdk::abi::internal::EncodableReturnType;
                        EncodableReturnType::encode(call_result)
                    },
                ),
                "Result" => {
                    // Extract the generic arguments
                    let args = match &segment.arguments {
                        syn::PathArguments::AngleBracketed(args) => args,
                        _ => {
                            emit_error!(
                                ty.span(),
                                "Expected Result to have generic arguments, found none"
                            );
                            return get_default_output(ty);
                        }
                    };

                    // Get the first generic argument (T in Result<T, E>)
                    if args.args.is_empty() {
                        emit_error!(
                            ty.span(),
                            "Expected Result to have at least one generic argument"
                        );
                        return get_default_output(ty);
                    }
                    if let syn::GenericArgument::Type(ok_type) = &args.args[0] {
                        get_default_output(ok_type)
                    } else {
                        emit_error!(
                            ty.span(),
                            "Expected Result to have a type as the first generic argument"
                        );
                        get_default_output(ty)
                    }
                }
                _ => get_default_output(ty),
            }
        }
    }
}

impl PublicImpl {
    pub fn impl_router(&self) -> syn::ItemImpl {
        let Self {
            self_ty,
            generic_params,
            where_clause,
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

        let fallback = call_special!(
            self,
            FnKind::Fallback { .. },
            "fallback",
            PublicFn::call_fallback
        );
        let fallback = fallback.unwrap_or_else(|| parse_quote!({ None }));

        let receive = call_special!(self, FnKind::Receive, "receive", PublicFn::call_receive);
        let receive = receive.unwrap_or_else(|| parse_quote!({ None }));

        let call_constructor = call_special!(
            self,
            FnKind::Constructor,
            "constructor",
            PublicFn::call_constructor
        );
        let constructor = call_constructor.unwrap_or_else(|| parse_quote!({ None }));

        let implements_routes = self.implements_routes();

        // Determine trait dynamic interface with associated types
        let iface = match &self.trait_ {
            Some(trait_) => {
                if !self.associated_types.is_empty() {
                    let assoc_types_formatted = self
                        .associated_types
                        .iter()
                        .map(|(name, value)| {
                            quote! { #name = #value }
                        })
                        .collect::<Vec<_>>();

                    &parse_quote! { dyn #trait_ < #(#assoc_types_formatted),* > }
                } else {
                    &parse_quote! { dyn #trait_ }
                }
            }
            None => self_ty,
        };

        parse_quote! {
            #[cfg(not(feature = "contract-client-gen"))]
            impl<S, #generic_params> #Router<S, #iface> for #self_ty
            where
                S: stylus_sdk::stylus_core::storage::TopLevelStorage + core::borrow::BorrowMut<Self> + stylus_sdk::stylus_core::ValueDenier + stylus_sdk::stylus_core::ConstructorGuard,
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
                            #(#implements_routes)*
                            None
                        }
                    }
                }

                #[inline(always)]
                fn fallback(storage: &mut S, input: &[u8]) -> Option<stylus_sdk::ArbResult> {
                    #fallback
                }

                #[inline(always)]
                fn receive(storage: &mut S) -> Option<Result<(), Vec<u8>>> {
                    #receive
                }

                #[inline(always)]
                fn constructor(storage: &mut S, input: &[u8]) -> Option<stylus_sdk::ArbResult> {
                    #constructor
                }
            }
        }
    }

    fn implements_routes(&self) -> impl Iterator<Item = syn::ExprIf> + '_ {
        let self_ty = self.self_ty.clone();
        self.implements.iter().map(move |ty| {
            parse_quote! {
                if let Some(result) = <#self_ty as #Router<S, dyn #ty>>::route(storage, selector, input) {
                    return Some(result);
                }
            }
        })
    }

    pub fn contract_client_gen(&self) -> proc_macro2::TokenStream {
        let client_funcs = self
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

            let (output_type, output_decoding) = get_output_type_and_decoding(&func.output);

            let function_selector = func.function_selector();

            quote! {
                pub fn #func_name(
                    &self,
                    host: &dyn stylus_sdk::stylus_core::host::Host,
                    context: impl #context,
                    #(#inputs,)*
                ) -> #output_type {
                    let inputs = <<(#(#inputs_types,)*) as #AbiType>::SolType as #SolType>::abi_encode_params(&(#(#inputs_names,)*));
                    use stylus_sdk::function_selector;
                    let mut calldata = Vec::from(#function_selector);
                    calldata.extend(inputs);
                    let call_result = #call(host, context, self.#STYLUS_CONTRACT_ADDRESS_FIELD, &calldata)?;
                    #output_decoding
                }
            }
        })
        .collect::<proc_macro2::TokenStream>();

        let struct_path = self.self_ty.clone();

        let mut output = quote! {
            #[cfg(feature = "contract-client-gen")]
            impl #struct_path {
                #client_funcs
            }
        };

        // If the impl does not implement a trait, we add a constructor for the contract client
        if self.trait_.is_none() {
            output.extend(quote! {
                #[cfg(feature = "contract-client-gen")]
                impl #struct_path {
                    pub fn new(address: stylus_sdk::alloy_primitives::Address) -> Self {
                        Self {
                            #STYLUS_CONTRACT_ADDRESS_FIELD: address,
                        }
                    }
                }
            });
        }

        output
    }
}

#[derive(Debug)]
pub enum FnKind {
    Function,
    Fallback { with_args: bool },
    Receive,
    Constructor,
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
    pub output: syn::ReturnType,
    pub output_span: Span,

    #[allow(dead_code)]
    pub extension: E,
}

impl<E: FnExtension> PublicFn<E> {
    pub fn function_selector(&self) -> syn::Expr {
        let sol_name = syn::LitStr::new(&self.sol_name.as_string(), self.sol_name.span());
        let arg_types = self.arg_types();
        parse_quote! {
            function_selector!(#sol_name #(, #arg_types )*)
        }
    }

    pub fn selector_name(&self) -> syn::Ident {
        syn::Ident::new(&format!("__SELECTOR_{}", self.name), self.name.span())
    }

    fn selector_value(&self) -> syn::Expr {
        let function_selector = self.function_selector();
        parse_quote! {
            u32::from_be_bytes(#function_selector)
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
                let args = match <#decode_inputs as #SolType>::abi_decode_params(input) {
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
                if let Err(err) = storage.deny_value(#name) {
                    return Some(Err(err));
                }
            })
        }
    }

    fn call_fallback(&self) -> syn::Stmt {
        let deny_value = self.deny_value();
        let name = &self.name;
        let storage_arg = self.storage_arg();
        let call: syn::Stmt = if matches!(self.kind, FnKind::Fallback { with_args: false }) {
            parse_quote! {
                return Some({
                    if let Err(err) = Self::#name(#storage_arg) {
                        Err(err)
                    } else {
                        Ok(Vec::new())
                    }
                });
            }
        } else {
            parse_quote! {
                return Some(Self::#name(#storage_arg input));
            }
        };
        parse_quote!({
            #deny_value
            #call
        })
    }

    fn call_receive(&self) -> syn::Stmt {
        let name = &self.name;
        let storage_arg = self.storage_arg();
        parse_quote! {
            return Some(Self::#name(#storage_arg));
        }
    }

    fn call_constructor(&self) -> syn::Stmt {
        let deny_value = self.deny_value();
        let name = &self.name;
        let decode_inputs = self.decode_inputs();
        let storage_arg = self.storage_arg();
        let expand_args = self.expand_args();
        let encode_output = self.encode_output();
        parse_quote!({
            use stylus_sdk::abi::{internal, internal::EncodableReturnType};
            #deny_value
            if let Err(e) = storage.check_constructor_slot() {
                return Some(Err(e));
            }
            let args = match <#decode_inputs as #SolType>::abi_decode_params(input) {
                Ok(args) => args,
                Err(err) => {
                    internal::failed_to_decode_arguments(err);
                    return Some(Err(Vec::new()));
                }
            };
            let result = Self::#name(#storage_arg #(#expand_args, )* );
            Some(#encode_output)
        })
    }
}

pub struct PublicFnArg<E: FnArgExtension> {
    pub ty: syn::Type,
    pub name: syn::Ident,
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
