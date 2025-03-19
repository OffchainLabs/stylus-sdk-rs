// Copyright 2022-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use cfg_if::cfg_if;
use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro_error::emit_error;
use quote::ToTokens;
use syn::{parse_macro_input, parse_quote, spanned::Spanned, ReturnType, Signature, Type};

use crate::{
    types::Purity,
    utils::{
        attrs::{check_attr_is_empty, consume_attr, consume_flag},
        split_item_impl_for_impl,
    },
};
use types::{
    FnArgExtension, FnExtension, FnKind, InterfaceExtension, PublicFn, PublicFnArg, PublicImpl,
};

mod attrs;
mod overrides;
mod types;

cfg_if! {
    if #[cfg(feature = "export-abi")] {
        mod export_abi;
        type Extension = export_abi::InterfaceAbi;
    } else {
        type Extension = ();
    }
}

/// Implementation of the [`#[public]`][crate::public] macro.
///
/// This implementation performs the following steps:
/// - Parse the input as [`syn::ItemImpl`]
/// - Generate AST items within a [`PublicImpl`]
/// - Expand those AST items into tokens for output
pub fn public(attr: TokenStream, input: TokenStream) -> TokenStream {
    check_attr_is_empty(attr);
    let mut item_impl = parse_macro_input!(input as syn::ItemImpl);
    let public_impl = PublicImpl::<Extension>::from(&mut item_impl);

    let mut output = item_impl.into_token_stream();
    public_impl.to_tokens(&mut output);
    output.into()
}

impl ToTokens for PublicImpl {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.impl_router().to_tokens(tokens);
        if self.trait_.is_none() {
            self.impl_override_checks().to_tokens(tokens);
            Extension::codegen(self).to_tokens(tokens);
        }
    }
}

impl From<&mut syn::ItemImpl> for PublicImpl {
    fn from(node: &mut syn::ItemImpl) -> Self {
        // parse inheritance from #[inherits(...)] attribute
        let mut inheritance = Vec::new();
        if let Some(inherits) = consume_attr::<attrs::Inherit>(&mut node.attrs, "inherit") {
            inheritance.extend(inherits.types);
        }

        // parse traits from #[implements(...)] attribute
        let mut implements = Vec::new();
        if let Some(attr) = consume_attr::<attrs::Implements>(&mut node.attrs, "implements") {
            implements.extend(attr.types);
        }

        // collect public functions
        let funcs = node
            .items
            .iter_mut()
            .filter_map(|item| match item {
                syn::ImplItem::Fn(func) => Some(PublicFn::from(func)),
                syn::ImplItem::Const(_) => {
                    emit_error!(item, "unsupported impl item");
                    None
                }
                _ => {
                    // allow other item types
                    None
                }
            })
            .collect();

        let (generic_params, self_ty, where_clause) = split_item_impl_for_impl(node);
        let trait_ = match &node.trait_ {
            Some((_, trait_, _)) => Some(trait_.clone()),
            _ => None,
        };

        #[allow(clippy::let_unit_value)]
        let extension = <Extension as InterfaceExtension>::build(node);
        Self {
            self_ty,
            generic_params,
            where_clause,
            trait_,
            inheritance,
            implements,
            funcs,
            extension,
        }
    }
}

impl<E: FnExtension> From<&mut syn::ImplItemFn> for PublicFn<E> {
    fn from(node: &mut syn::ImplItemFn) -> Self {
        // parse attributes
        let payable = consume_flag(&mut node.attrs, "payable");
        let selector_override =
            consume_attr::<attrs::Selector>(&mut node.attrs, "selector").map(|s| s.value.value());
        let fallback = consume_flag(&mut node.attrs, "fallback");
        let receive = consume_flag(&mut node.attrs, "receive");
        let constructor = consume_flag(&mut node.attrs, "constructor");

        let kind = if fallback {
            // Fallback functions may have two signatures, either
            // with input calldata and output bytes, or no input and output.
            // We check if the signature is correct for these two cases
            // early and emit a proc macro error if it is not the case.
            check_fallback_signature(node.sig.clone());
            FnKind::Fallback {
                with_args: node.sig.inputs.len() > 1,
            }
        } else if receive {
            FnKind::Receive
        } else if constructor {
            FnKind::Constructor
        } else {
            FnKind::Function
        };

        let num_specials = (fallback as i8) + (constructor as i8) + (receive as i8);
        if num_specials > 1 {
            emit_error!(
                node.span(),
                "function can be only one of fallback, receive or constructor"
            );
        }
        if num_specials > 0 && selector_override.is_some() {
            emit_error!(
                node.span(),
                "fallback, receive, and constructor can't have custom selector"
            );
        }

        // name for generated rust, and solidity abi
        let name = node.sig.ident.clone();
        for special_name in ["receive", "fallback", "constructor"] {
            if matches!(kind, FnKind::Function) && name == special_name {
                emit_error!(
                    node.span(),
                    format!("{special_name} function can only be defined using the #[{special_name}] attribute")
                );
            }
        }

        let sol_name = syn_solidity::SolIdent::new(
            &selector_override.unwrap_or(name.to_string().to_case(Case::Camel)),
        );

        // determine state mutability
        let (inferred_purity, has_self) = Purity::infer(node);
        let purity = if payable || matches!(kind, FnKind::Receive) {
            Purity::Payable
        } else {
            inferred_purity
        };

        let mut args = node.sig.inputs.iter();
        if inferred_purity > Purity::Pure {
            // skip self or storage argument
            args.next();
        }
        let inputs = match kind {
            FnKind::Function | FnKind::Constructor => args.map(PublicFnArg::from).collect(),
            _ => Vec::new(),
        };
        let input_span = node.sig.inputs.span();

        let output = match &node.sig.output {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, ty) => Some(*ty.clone()),
        };
        let output_span = output
            .as_ref()
            .map(Spanned::span)
            .unwrap_or(node.sig.output.span());

        let extension = E::build(node);
        Self {
            name,
            sol_name,
            purity,
            inferred_purity,
            kind,

            has_self,
            inputs,
            input_span,
            output_span,

            extension,
        }
    }
}

impl<E: FnArgExtension> From<&syn::FnArg> for PublicFnArg<E> {
    fn from(node: &syn::FnArg) -> Self {
        match node {
            syn::FnArg::Typed(pat_type) => {
                let extension = E::build(node);
                Self {
                    ty: *pat_type.ty.clone(),
                    extension,
                }
            }
            _ => unreachable!(),
        }
    }
}

fn check_fallback_signature(sig: Signature) {
    let has_input_args = sig.inputs.len() > 0;
    let has_output = !matches!(sig.output, ReturnType::Default);

    if has_input_args {
        // Fallback functions with input args must return Result<Vec<u8>, Vec<u8>>
        let expected: Type = parse_quote! { Result<Vec<u8>, Vec<u8>> };

        match &sig.output {
            ReturnType::Default => {
                emit_error!(
                    sig.output.span(),
                    "fallback function with input args must have output args"
                );
            }
            ReturnType::Type(_, ty) => {
                if **ty != expected {
                    emit_error!(
                        ty.span(),
                        "fallback function with input args must output Result<Vec<u8>, Vec<u8>>"
                    );
                }
            }
        }
    } else {
        // Fallback functions without input args must not have output
        if has_output {
            emit_error!(
                sig.output.span(),
                "fallback function without input args must have no output args"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::types::{FnKind, PublicImpl};

    #[test]
    fn test_public_consumes_inherit() {
        let mut impl_item = parse_quote! {
            #[derive(Debug)]
            #[inherit(Parent)]
            impl Contract {
            }
        };
        let _public = PublicImpl::from(&mut impl_item);
        assert_eq!(impl_item.attrs, vec![parse_quote! { #[derive(Debug)] }]);
    }

    #[test]
    fn test_public_consumes_payable() {
        let mut impl_item = parse_quote! {
            #[derive(Debug)]
            impl Contract {
                #[payable]
                #[other]
                fn func() {}
            }
        };
        let _public = PublicImpl::from(&mut impl_item);
        let syn::ImplItem::Fn(syn::ImplItemFn { attrs, .. }) = &impl_item.items[0] else {
            unreachable!();
        };
        assert_eq!(attrs, &vec![parse_quote! { #[other] }]);
    }

    #[test]
    fn test_public_consumes_constructor() {
        let mut impl_item = parse_quote! {
            #[derive(Debug)]
            impl Contract {
                #[constructor]
                fn func(&mut self, val: U256) {}
            }
        };
        let public = PublicImpl::from(&mut impl_item);
        assert!(matches!(public.funcs[0].kind, FnKind::Constructor));
        let syn::ImplItem::Fn(syn::ImplItemFn { attrs, .. }) = &impl_item.items[0] else {
            unreachable!();
        };
        assert!(attrs.is_empty());
    }
}
