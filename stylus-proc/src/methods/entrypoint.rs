// Copyright 2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use cfg_if::cfg_if;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, Item};

pub fn entrypoint(attr: TokenStream, input: TokenStream) -> TokenStream {
    let input: Item = parse_macro_input!(input);

    if !attr.is_empty() {
        error!(Span::mixed_site(), "this macro is not configurable");
    }

    let mut output = quote! { #input };

    let user = match input {
        Item::Struct(input) => {
            let name = &input.ident;
            let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

            output.extend(quote!{
                unsafe impl #impl_generics stylus_sdk::storage::TopLevelStorage for #name #ty_generics #where_clause {}

                fn entrypoint(input: alloc::vec::Vec<u8>) -> stylus_sdk::ArbResult {
                    use stylus_sdk::{abi::Router, alloy_primitives::U256, console, hex, storage::StorageType};
                    use core::convert::TryInto;
                    use alloc::vec;

                    if input.len() < 4 {
                        console!("calldata too short: {}", hex::encode(input));
                        return Err(vec![]);
                    }
                    let selector = u32::from_be_bytes(TryInto::try_into(&input[..4]).unwrap());
                    let mut storage = unsafe { <#name as StorageType>::new(U256::ZERO, 0) };
                    match <#name as Router<_>>::route(&mut storage, selector, &input[4..]) {
                        Some(res) => res,
                        None => {
                            console!("unknown method selector: {selector:08x}");
                            Err(vec![])
                        },
                    }
                }
            });

            if cfg!(feature = "export-abi") {
                output.extend(quote! {
                    pub fn main() {
                        stylus_sdk::abi::export::print_abi::<#name>();
                    }
                });
            }

            Ident::new("entrypoint", name.span())
        }
        Item::Fn(input) => input.sig.ident.clone(),
        _ => error!(input, "not a struct or fn"),
    };

    // revert on reentrancy unless explicitly enabled
    cfg_if! {
        if #[cfg(feature = "reentrant")] {
            let deny_reentrant = quote! {};
        } else {
            let deny_reentrant = quote! {
                if stylus_sdk::msg::reentrant() {
                    return 1; // revert
                }
            };
        }
    }

    output.extend(quote! {
        #[no_mangle]
        pub unsafe fn mark_used() {
            stylus_sdk::evm::pay_for_memory_grow(0);
            panic!();
        }

        #[no_mangle]
        pub extern "C" fn user_entrypoint(len: usize) -> usize {
            #deny_reentrant

            let input = stylus_sdk::contract::args(len);
            let (data, status) = match #user(input) {
                Ok(data) => (data, 0),
                Err(data) => (data, 1),
            };
            unsafe { stylus_sdk::storage::StorageCache::flush() };
            stylus_sdk::contract::output(&data);
            status
        }
    });

    output.into()
}
