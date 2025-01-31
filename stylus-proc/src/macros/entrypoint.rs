// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use cfg_if::cfg_if;
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::{abort, emit_error};
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
};

use crate::consts::{ASSERT_OVERRIDES_FN, STRUCT_ENTRYPOINT_FN};

/// Implementation for the [`#[entrypoint]`][crate::entrypoint] macro.
pub fn entrypoint(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    if !attr.is_empty() {
        emit_error!(Span::mixed_site(), "this macro is not configurable");
    }

    let entrypoint: Entrypoint = parse_macro_input!(input);
    entrypoint.into_token_stream().into()
}

struct Entrypoint {
    kind: EntrypointKind,
    mark_used_fn: syn::ItemFn,
    user_entrypoint_fn: syn::ItemFn,
}
impl Parse for Entrypoint {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let item: syn::Item = input.parse()?;
        let kind = match item {
            syn::Item::Fn(item) => EntrypointKind::Fn(EntrypointFn { item }),
            syn::Item::Struct(item) => EntrypointKind::Struct(EntrypointStruct {
                top_level_storage_impl: top_level_storage_impl(&item),
                struct_entrypoint_fn: struct_entrypoint_fn(&item.ident),
                assert_overrides_const: assert_overrides_const(&item.ident),
                print_abi_fn: print_abi_fn(&item.ident),
                item,
            }),
            _ => abort!(item, "not a struct or fn"),
        };

        Ok(Self {
            user_entrypoint_fn: user_entrypoint_fn(kind.entrypoint_fn_name()),
            mark_used_fn: mark_used_fn(),
            kind,
        })
    }
}

impl ToTokens for Entrypoint {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.kind.to_tokens(tokens);
        self.mark_used_fn.to_tokens(tokens);
        self.user_entrypoint_fn.to_tokens(tokens);
    }
}

enum EntrypointKind {
    Fn(EntrypointFn),
    Struct(EntrypointStruct),
}

impl EntrypointKind {
    fn entrypoint_fn_name(&self) -> Ident {
        match self {
            EntrypointKind::Fn(EntrypointFn { item }) => item.sig.ident.clone(),
            EntrypointKind::Struct(EntrypointStruct { item, .. }) => {
                let mut ident = STRUCT_ENTRYPOINT_FN.as_ident();
                ident.set_span(item.ident.span());
                ident
            }
        }
    }
}

impl ToTokens for EntrypointKind {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            EntrypointKind::Fn(inner) => inner.to_tokens(tokens),
            EntrypointKind::Struct(inner) => inner.to_tokens(tokens),
        }
    }
}

struct EntrypointFn {
    item: syn::ItemFn,
}

impl ToTokens for EntrypointFn {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.item.to_tokens(tokens);
    }
}

struct EntrypointStruct {
    item: syn::ItemStruct,
    top_level_storage_impl: syn::ItemImpl,
    struct_entrypoint_fn: syn::ItemFn,
    assert_overrides_const: syn::ItemConst,
    print_abi_fn: Option<syn::ItemFn>,
}

impl ToTokens for EntrypointStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.item.to_tokens(tokens);
        self.top_level_storage_impl.to_tokens(tokens);
        self.struct_entrypoint_fn.to_tokens(tokens);
        self.assert_overrides_const.to_tokens(tokens);
        self.print_abi_fn.to_tokens(tokens);
    }
}

fn top_level_storage_impl(item: &syn::ItemStruct) -> syn::ItemImpl {
    let name = &item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    parse_quote! {
        unsafe impl #impl_generics stylus_sdk::stylus_core::storage::TopLevelStorage for #name #ty_generics #where_clause {}
    }
}

fn struct_entrypoint_fn(name: &Ident) -> syn::ItemFn {
    parse_quote! {
        fn #STRUCT_ENTRYPOINT_FN(input: alloc::vec::Vec<u8>) -> stylus_sdk::ArbResult {
            stylus_sdk::abi::router_entrypoint::<#name, #name>(input, stylus_sdk::host::VM{})
        }
    }
}

fn assert_overrides_const(name: &Ident) -> syn::ItemConst {
    parse_quote! {
        const _: () = {
            <#name>::#ASSERT_OVERRIDES_FN();
        };
    }
}

fn mark_used_fn() -> syn::ItemFn {
    parse_quote! {
        #[cfg(target_arch = "wasm32")]
        #[no_mangle]
        pub unsafe fn mark_used() {
            // let host = stylus_sdk::host::VM(stylus_sdk::host::WasmVM{});
            stylus_sdk::evm::pay_for_memory_grow(0);
            panic!();
        }
    }
}

fn user_entrypoint_fn(user_fn: Ident) -> syn::ItemFn {
    let deny_reentrant = deny_reentrant();
    parse_quote! {
        #[cfg(target_arch = "wasm32")]
        #[no_mangle]
        pub extern "C" fn user_entrypoint(len: usize) -> usize {
            // let host = stylus_sdk::host::VM(stylus_sdk::host::WasmVM{});
            #deny_reentrant
            stylus_sdk::evm::pay_for_memory_grow(0);

            let input = stylus_sdk::contract::args(len);
            let (data, status) = match #user_fn(input) {
                Ok(data) => (data, 0),
                Err(data) => (data, 1),
            };
            // host.flush_cache(false /* do not clear */);
            // host.write_result(&data);
            unsafe { stylus_sdk::storage::StorageCache::flush() };
            stylus_sdk::contract::output(&data);
            status
        }
    }
}

/// Revert on reentrancy unless explicitly enabled
fn deny_reentrant() -> Option<syn::ExprIf> {
    cfg_if! {
        if #[cfg(feature = "reentrant")] {
            None
        } else {
            Some(parse_quote! {
                if stylus_sdk::msg::reentrant() {
                    return 1; // revert
                }
            })
        }
    }
}

fn print_abi_fn(ident: &syn::Ident) -> Option<syn::ItemFn> {
    let _ = ident;
    cfg_if! {
        if #[cfg(feature = "export-abi")] {
            Some(parse_quote! {
                pub fn print_abi(license: &str, pragma: &str) {
                    stylus_sdk::abi::export::print_abi::<#ident>(license, pragma);
                }
            })
        } else {
            None
        }
    }
}
