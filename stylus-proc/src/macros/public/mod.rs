// Copyright 2022-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use cfg_if::cfg_if;
use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro_error::emit_error;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, spanned::Spanned};

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

    let mut output = quote! {
        #[cfg(not(feature = "contract-client-gen"))]
    };
    output.extend(item_impl.into_token_stream());

    public_impl.to_tokens(&mut output);

    output.into()
}

impl ToTokens for PublicImpl {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.contract_client_gen());
        self.impl_router().to_tokens(tokens);
        if self.trait_.is_none() {
            Extension::codegen(self).to_tokens(tokens);
        }
    }
}

impl From<&mut syn::ItemImpl> for PublicImpl {
    fn from(node: &mut syn::ItemImpl) -> Self {
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

        // Extract associated types from the impl items
        let mut associated_types = Vec::new();
        for item in &node.items {
            if let syn::ImplItem::Type(type_item) = item {
                associated_types.push((type_item.ident.clone(), type_item.ty.clone()));
            }
        }

        #[allow(clippy::let_unit_value)]
        let extension = <Extension as InterfaceExtension>::build(node);
        Self {
            self_ty,
            generic_params,
            where_clause,
            trait_,
            implements,
            funcs,
            associated_types,
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
        let (sol_name, name_err) = verify_sol_name(&kind, name.to_string(), selector_override);
        if let Some(err) = name_err {
            emit_error!(node.span(), err);
        }
        let sol_name = syn_solidity::SolIdent::new(&sol_name);

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
            output: node.sig.output.clone(),
            output_span,

            extension,
        }
    }
}

impl<E: FnArgExtension> From<&syn::FnArg> for PublicFnArg<E> {
    fn from(node: &syn::FnArg) -> Self {
        match node {
            syn::FnArg::Typed(pat_type) => match &*pat_type.pat {
                syn::Pat::Ident(pat_ident) => Self {
                    name: pat_ident.ident.clone(),
                    ty: *pat_type.ty.clone(),
                    extension: E::build(node),
                },
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
}

/// Returns the Solidity name used for routing and an error string if the name doesn't match the function kind.
fn verify_sol_name(
    kind: &FnKind,
    name: String,
    selector_override: Option<String>,
) -> (String, Option<String>) {
    let name = selector_override.unwrap_or(name.to_case(Case::Camel));
    let name_low = name.to_lowercase();
    let err_kind = if name_low == "receive" && !matches!(kind, FnKind::Receive) {
        Some("receive")
    } else if name_low == "fallback" && !matches!(kind, FnKind::Fallback { .. }) {
        Some("fallback")
    } else if (name_low == "constructor" || name_low == "stylus_constructor")
        && !matches!(kind, FnKind::Constructor)
    {
        Some("constructor")
    } else {
        None
    };
    let err = err_kind.map(|kind_name| {
        format!("{kind_name} function can only be defined using the corresponding attribute")
    });
    (name, err)
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::{
        types::{FnKind, PublicImpl},
        verify_sol_name,
    };

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

    #[test]
    fn test_verify_sol_name() {
        let cases = vec![
            ("foo", None, "foo", false),
            ("foo_bar", None, "fooBar", false),
            ("foo_baz", Some("fooBar"), "fooBar", false),
            ("foo_baz", Some("fooBAR"), "fooBAR", false),
            ("receive", None, "receive", true),
            ("re_ceive", None, "reCeive", true),
            ("foo", Some("RECEIVE"), "RECEIVE", true),
        ];
        for (name, selector_override, expected_sol_name, has_err) in cases {
            let kind = FnKind::Function;
            let (sol_name, err) =
                verify_sol_name(&kind, name.to_owned(), selector_override.map(String::from));
            assert_eq!(sol_name, expected_sol_name);
            assert_eq!(err.is_some(), has_err);
        }
    }
}
