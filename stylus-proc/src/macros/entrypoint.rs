// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use cfg_if::cfg_if;
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::{abort, emit_error};
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    token::Comma,
};

use crate::consts::{STRUCT_ENTRYPOINT_FN, STYLUS_CONTRACT_ADDRESS_FIELD};

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

pub fn appends_stylus_contract_address(item_struct: &mut syn::ItemStruct) -> syn::Result<()> {
    let new_field: syn::Field = parse_quote! { #STYLUS_CONTRACT_ADDRESS_FIELD: Address };

    match &mut item_struct.fields {
        syn::Fields::Named(named_fields) => {
            named_fields.named.push(new_field);
            Ok(())
        }
        syn::Fields::Unit => {
            // Transform unit struct directly into a named struct with the new field
            let mut named_fields = Punctuated::<syn::Field, Comma>::new();
            named_fields.push(new_field);

            // Replace Fields::Unit with Fields::Named
            item_struct.fields = syn::Fields::Named(parse_quote! { { #named_fields } });
            item_struct.semi_token = None; // Named structs do not have a semicolon at the end
            Ok(())
       }
       syn::Fields::Unnamed(_) => {
            Err(syn::Error::new_spanned(
                &item_struct.ident,
                "[entrypoint] only supports named and unit structs.",
            ))
        }
    }
}

struct Entrypoint {
    kind: EntrypointKind,
    user_entrypoint_fn: Option<syn::ItemFn>,
}
impl Parse for Entrypoint {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut item: syn::Item = input.parse()?;
        let kind = match item {
            syn::Item::Fn(item) => EntrypointKind::Fn(EntrypointFn { item }),
            syn::Item::Struct(ref mut item) => {
                if cfg!(feature = "contract-client-gen") {
                    match appends_stylus_contract_address(item) {
                        Err(e) => return Err(e),
                        Ok(_) => (),
                    }
                }

                EntrypointKind::Struct(EntrypointStruct {
                    top_level_storage_impl: top_level_storage_impl(&item),
                    struct_entrypoint_fn: struct_entrypoint_fn(&item.ident),
                    print_from_args_fn: print_from_args_fn(&item.ident),
                    item: item.clone(),
                })
            }
            _ => abort!(item, "not a struct or fn"),
        };

        Ok(Self {
            user_entrypoint_fn: user_entrypoint_fn(kind.entrypoint_fn_name()),
            kind,
        })
    }
}

impl ToTokens for Entrypoint {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.kind.to_tokens(tokens);
        self.user_entrypoint_fn.to_tokens(tokens);
    }
}

#[allow(clippy::large_enum_variant)]
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
    print_from_args_fn: Option<syn::ItemFn>,
}

impl ToTokens for EntrypointStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.item.to_tokens(tokens);
        self.top_level_storage_impl.to_tokens(tokens);
        self.struct_entrypoint_fn.to_tokens(tokens);
        self.print_from_args_fn.to_tokens(tokens);
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
        fn #STRUCT_ENTRYPOINT_FN(input: alloc::vec::Vec<u8>, host: stylus_sdk::host::VM) -> stylus_sdk::ArbResult {
            stylus_sdk::abi::router_entrypoint::<#name, #name>(input, host)
        }
    }
}

fn user_entrypoint_fn(user_fn: Ident) -> Option<syn::ItemFn> {
    let _ = user_fn;
    cfg_if::cfg_if! {
        if #[cfg(feature = "stylus-test")] {
            None
        } else {
            let deny_reentrant = deny_reentrant();
            Some(parse_quote! {
                #[no_mangle]
                pub extern "C" fn user_entrypoint(len: usize) -> usize {
                    let host = stylus_sdk::host::VM(stylus_sdk::host::WasmVM{});
                    #deny_reentrant

                    // The following call is a noop to ensure that pay_for_memory_grow is
                    // referenced by the Stylus contract. Later, when the contract is activated,
                    // Nitro will automatically add the calls pay_for_memory_grow when memory is
                    // dynamically allocated. If we do not add this call here, the calls added by
                    // Nitro will not work and activation will fail. This call costs 8700 Ink,
                    // which is less than 1 Gas.
                    host.pay_for_memory_grow(0);

                    let input = host.read_args(len);
                    let (data, status) = match #user_fn(input, host.clone()) {
                        Ok(data) => (data, 0),
                        Err(data) => (data, 1),
                    };
                    host.flush_cache(false /* do not clear */);
                    host.write_result(&data);
                    status
                }
            })
        }
    }
}

/// Revert on reentrancy unless explicitly enabled
#[cfg(not(feature = "stylus-test"))]
fn deny_reentrant() -> Option<syn::ExprIf> {
    cfg_if! {
        if #[cfg(feature = "reentrant")] {
            None
        } else {
            Some(parse_quote! {
                if host.msg_reentrant() {
                    return 1; // revert
                }
            })
        }
    }
}

fn print_from_args_fn(ident: &syn::Ident) -> Option<syn::ItemFn> {
    let _ = ident;
    cfg_if! {
        if #[cfg(feature = "export-abi")] {
            Some(parse_quote! {
                pub fn print_from_args() {
                    stylus_sdk::abi::export::print_from_args::<#ident>();
                }
            })
        } else {
            None
        }
    }
}
