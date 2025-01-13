// Copyright 2022-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use cfg_if::cfg_if;
use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro_error::emit_error;
use quote::ToTokens;
use syn::{parse_macro_input, parse_quote, spanned::Spanned, ItemImpl, TypeParamBound};

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
    let item_impl = parse_macro_input!(input as syn::ItemImpl);

    let ItemImpl {
        mut generics,
        self_ty,
        items,
        ..
    } = item_impl;

    let is_stylus_host_path = |bound: &TypeParamBound| -> bool {
        if let syn::TypeParamBound::Trait(trait_bound) = bound {
            // Check if the path is stylus_sdk::host::Host
            let segments = &trait_bound.path.segments;
            segments.len() == 3
                && segments[0].ident == "stylus_sdk"
                && segments[1].ident == "host"
                && segments[2].ident == "Host"
        } else {
            false
        }
    };
    // Check if H exists and has Host bound in params.
    let h_with_host = generics.params.iter().any(|param| {
        if let syn::GenericParam::Type(type_param) = param {
            if type_param.ident == "H" {
                return type_param.bounds.iter().any(is_stylus_host_path);
            }
            false
        } else {
            false
        }
    });

    // Check if H exists in where clauses.
    let h_in_where = if let Some(where_clause) = &generics.where_clause {
        where_clause.predicates.iter().any(|pred| {
            if let syn::WherePredicate::Type(pred_type) = pred {
                if let syn::Type::Path(type_path) = &pred_type.bounded_ty {
                    if type_path.path.segments.len() == 1 && type_path.path.segments[0].ident == "H"
                    {
                        return pred_type.bounds.iter().any(is_stylus_host_path);
                    }
                }
                false
            } else {
                false
            }
        })
    } else {
        false
    };
    if !h_with_host && !h_in_where {
        // Add H: Host to generics
        let host_param: syn::GenericParam = parse_quote!(H: stylus_sdk::host::Host);
        generics.params.push(host_param);
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let type_name = if let syn::Type::Path(type_path) = self_ty.as_ref() {
        if let Some(last_segment) = type_path.path.segments.last() {
            &last_segment.ident
        } else {
            return syn::Error::new_spanned(self_ty, "Expected a named type")
                .to_compile_error()
                .into();
        }
    } else {
        return syn::Error::new_spanned(self_ty, "Expected a named type")
            .to_compile_error()
            .into();
    };

    let mut host_injected_impl: ItemImpl = parse_quote! {
        impl #impl_generics #type_name #ty_generics #where_clause {
            #(#items)*
        }
    };

    let public_impl = PublicImpl::<Extension>::from(&mut host_injected_impl);
    let mut output = host_injected_impl.into_token_stream();
    public_impl.to_tokens(&mut output);
    output.into()
}

impl ToTokens for PublicImpl {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.impl_router().to_tokens(tokens);
        self.impl_override_checks().to_tokens(tokens);
        Extension::codegen(self).to_tokens(tokens);
    }
}

impl From<&mut syn::ItemImpl> for PublicImpl {
    fn from(node: &mut syn::ItemImpl) -> Self {
        // parse inheritance from #[inherits(...)] attribute
        let mut inheritance = Vec::new();
        if let Some(inherits) = consume_attr::<attrs::Inherit>(&mut node.attrs, "inherit") {
            inheritance.extend(inherits.types);
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
        #[allow(clippy::let_unit_value)]
        let extension = <Extension as InterfaceExtension>::build(node);
        Self {
            self_ty,
            generic_params,
            where_clause,
            inheritance,
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

        let kind = match (fallback, receive) {
            (true, false) => {
                // Fallback functions may have two signatures, either
                // with input calldata and output bytes, or no input and output.
                // node.sig.
                let has_inputs = node.sig.inputs.len() > 1;
                if has_inputs {
                    FnKind::FallbackWithArgs
                } else {
                    FnKind::FallbackNoArgs
                }
            }
            (false, true) => FnKind::Receive,
            (false, false) => FnKind::Function,
            (true, true) => {
                emit_error!(node.span(), "function cannot be both fallback and receive");
                FnKind::Function
            }
        };

        // name for generated rust, and solidity abi
        let name = node.sig.ident.clone();

        if matches!(kind, FnKind::Function) && (name == "receive" || name == "fallback") {
            emit_error!(
                node.span(),
                "receive and/or fallback functions can only be defined using the #[receive] or "
                    .to_string()
                    + "#[fallback] attribute instead of names",
            );
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
            FnKind::Function => args.map(PublicFnArg::from).collect(),
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

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::types::PublicImpl;

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
}
