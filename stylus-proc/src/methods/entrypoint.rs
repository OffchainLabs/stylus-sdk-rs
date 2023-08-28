// Copyright 2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, ItemFn, ItemStruct, LitBool, Result, Token,
};

pub fn derive_entrypoint(input: TokenStream) -> TokenStream {
    let input: ItemStruct = parse_macro_input!(input);
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut output = quote! {
        unsafe impl #impl_generics stylus_sdk::storage::TopLevelStorage for #ident #ty_generics #where_clause {}

        #[entrypoint]
        fn entrypoint(input: alloc::vec::Vec<u8>) -> stylus_sdk::ArbResult {
            use stylus_sdk::{abi::Router, alloy_primitives::U256, console, hex, storage::StorageType};
            use core::convert::TryInto;
            use alloc::vec;

            if input.len() < 4 {
                console!("calldata too short: {}", hex::encode(input));
                return Err(vec![]);
            }
            let selector = u32::from_be_bytes(TryInto::try_into(&input[..4]).unwrap());
            let mut storage = unsafe { <#ident as StorageType>::new(U256::ZERO, 0) };
            match <#ident as Router<_>>::route(&mut storage, selector, &input[4..]) {
                Some(res) => res,
                None => {
                    console!("unknown method selector: {selector:08x}");
                    Err(vec![])
                },
            }
        }
    };

    if cfg!(feature = "export-abi") {
        output.extend(quote! {
            fn main() {
                stylus_sdk::abi::export::print_abi::<#ident>();
            }
        });
    }
    output.into()
}

pub fn entrypoint(attr: TokenStream, input: TokenStream) -> TokenStream {
    if let Ok(ItemStruct { .. }) = syn::parse_macro_input::parse(input.clone()) {
        return derive_entrypoint(input);
    }

    let input: ItemFn = parse_macro_input!(input);
    let args: EntrypointArgs = parse_macro_input!(attr);

    let name = &input.sig.ident;
    let allow_reentrancy = args.allow_reentrancy;

    quote! {
        #input

        #[no_mangle]
        pub unsafe fn mark_used() {
            stylus_sdk::evm::memory_grow(0);
            panic!();
        }

        #[no_mangle]
        pub extern "C" fn user_entrypoint(len: usize) -> usize {
            if !#allow_reentrancy && stylus_sdk::msg::reentrant() {
                return 1; // revert on reentrancy
            }
            if #allow_reentrancy {
                unsafe { stylus_sdk::call::opt_into_reentrancy() };
            }

            let input = stylus_sdk::contract::args(len);
            let (data, status) = match #name(input) {
                Ok(data) => (data, 0),
                Err(data) => (data, 1),
            };
            stylus_sdk::storage::StorageCache::flush();
            stylus_sdk::contract::output(&data);
            status
        }
    }
    .into()
}

struct EntrypointArgs {
    allow_reentrancy: bool,
}

impl Parse for EntrypointArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut allow_reentrancy = false;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            let _: Token![=] = input.parse()?;

            match ident.to_string().as_str() {
                "allow_reentrancy" => {
                    let lit: LitBool = input.parse()?;
                    allow_reentrancy = lit.value;
                }
                _ => error!(@ident, "Unknown entrypoint attribute"),
            }
        }
        Ok(Self { allow_reentrancy })
    }
}
