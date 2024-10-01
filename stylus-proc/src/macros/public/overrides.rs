// Copyright 2022-2024, Offchain Labs, Inc.
// use crate::consts::{ALLOW_OVERRIDE_FN, ASSERT_OVERRIDES_FN};
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Ensure that public functions follow safe override rules.

use proc_macro2::Span;
use syn::parse_quote;

use super::types::{FnExtension, PublicFn, PublicImpl};
use crate::consts::{ALLOW_OVERRIDE_FN, ASSERT_OVERRIDES_FN};

impl PublicImpl {
    pub fn impl_override_checks(&self) -> syn::ItemImpl {
        let Self {
            self_ty,
            generic_params,
            where_clause,
            ..
        } = self;
        let selector_consts = self
            .funcs
            .iter()
            .map(PublicFn::selector_const)
            .collect::<Vec<_>>();
        let override_arms = self.funcs.iter().map(PublicFn::override_arm);
        let inheritance_overrides = self.inheritance_overrides();
        let override_checks = self.override_checks();
        parse_quote! {
            impl<#generic_params> #self_ty where #where_clause {
                /// Whether or not to allow overriding a selector by a child contract and method with
                /// the given purity. This is currently implemented as a hidden function to allow it to
                /// be `const`. A trait would be better, but `const` is not currently supported for
                /// trait fns.
                #[doc(hidden)]
                pub const fn #ALLOW_OVERRIDE_FN(selector: u32, purity: stylus_sdk::methods::Purity) -> bool {
                    use stylus_sdk::function_selector;

                    #(#selector_consts)*
                    if !match selector {
                        #(#override_arms)*
                        _ => true
                    } { return false; }
                    #(#inheritance_overrides)*
                    true
                }

                /// Check the functions defined in an entrypoint for valid overrides.
                #[doc(hidden)]
                pub const fn #ASSERT_OVERRIDES_FN() {
                    use stylus_sdk::function_selector;

                    #(#selector_consts)*
                    #(#override_checks)*
                }
            }
        }
    }

    fn inheritance_overrides(&self) -> impl Iterator<Item = syn::ExprIf> + '_ {
        self.inheritance.iter().map(|ty| {
            parse_quote! {
                if !<#ty>::#ALLOW_OVERRIDE_FN(selector, purity) {
                    return false;
                }
            }
        })
    }

    fn override_checks(&self) -> impl Iterator<Item = syn::Stmt> + '_ {
        self.funcs
            .iter()
            .map(|func| func.assert_override(&self.self_ty))
            .chain(self.inheritance.iter().map(|ty| {
                parse_quote! {
                    <#ty>::#ASSERT_OVERRIDES_FN();
                }
            }))
    }
}

impl<E: FnExtension> PublicFn<E> {
    fn override_arm(&self) -> syn::Arm {
        let constant = self.selector_name();
        let purity = self.purity.as_path();
        parse_quote! {
            #[allow(non_upper_case_globals)]
            #constant => #purity.allow_override(purity),
        }
    }

    fn override_error(&self) -> syn::LitStr {
        syn::LitStr::new(
            &format!(
                "function {} cannot be overriden with function marked {:?}",
                self.name, self.purity,
            ),
            Span::mixed_site(),
        )
    }

    fn assert_override(&self, self_ty: &syn::Type) -> syn::Stmt {
        let purity = self.purity.as_path();
        let selector_name = self.selector_name();
        let error = self.override_error();
        parse_quote! {
            assert!(<#self_ty>::#ALLOW_OVERRIDE_FN(#selector_name, #purity), #error);
        }
    }
}
