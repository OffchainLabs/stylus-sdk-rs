// Copyright 2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

pub fn derive_entrypoint(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut output = quote! {
        unsafe impl #impl_generics stylus_sdk::storage::TopLevelStorage for #ident #ty_generics #where_clause {}

        fn entrypoint(input: Vec<u8>) -> stylus_sdk::ArbResult {
            use stylus_sdk::{abi::Router, alloy_primitives::U256, console, hex, storage::StorageType};
            use std::convert::TryInto;

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

        stylus_sdk::entrypoint!(entrypoint);
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
