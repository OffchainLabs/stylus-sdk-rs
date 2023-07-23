// Copyright 2022-2023, Offchain Labs, Inc.
// For license information, see https://github.com/OffchainLabs/nitro/blob/master/LICENSE

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote, ItemStruct, Type};

#[proc_macro_attribute]
pub fn storage(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as ItemStruct);

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    macro_rules! error {
        ($tokens:expr, $($msg:expr),+) => {{
            let error = syn::Error::new_spanned($tokens, format!($($msg),+));
            return error.to_compile_error().into();
        }};
    }

    let mut accessors = quote! {};
    let mut init = quote! {};

    for field in &mut input.fields {
        // deny complex types
        let Type::Path(ty) = &mut field.ty else {
            error!(&field, "Type not supported for EVM state storage");
        };

        let path = &ty.path.segments.last().unwrap().ident;
        let not_supported = format!("Type `{path}` not supported for EVM state storage");

        match path.to_string().as_str() {
            x @ ("u8" | "u16" | "u32" | "u64" | "u128" | "i8" | "i16" | "i32" | "i64" | "i128") => {
                error!(
                    &field,
                    "{not_supported}. Instead try `{}`.",
                    x.to_uppercase()
                );
            }
            "usize" => error!(&field, "{not_supported}. Instead try `U32`."),
            "isize" => error!(&field, "{not_supported}. Instead try `I32`."),

            // supported types
            "Address" => *ty = parse_quote! { stylus_sdk::storage::AddressAcc },
            "BlockHash" => *ty = parse_quote! { stylus_sdk::storage::BlockHashAcc },
            _ => {}
            /*"BlockNumber" => {}

            _x @ ("U0" | "U1" | "U8" | "U16" | "U32" | "U64" | "U128" | "U160" | "U192"
            | "U256" | "U320" | "U384" | "U448" | "U512" | "U1024" | "U2048" | "U4096") => {}
            _x @ ("I0" | "I1" | "I8" | "I16" | "I32" | "I64" | "I128" | "I160" | "I192"
            | "I256" | "I512") => {}
            _x @ ("B0" | "B16" | "B32" | "B64" | "B96" | "B128" | "B160" | "B192" | "B224"
            | "B256" | "B512" | "B1024" | "B2048") => {}
            _ => {}*/
        }

        let Some(ident) = &field.ident else {
            continue;
        };

        let get_ident = format_ident!("get_{}", ident);
        let set_ident = format_ident!("set_{}", ident);

        accessors.extend(quote! {
            pub fn #get_ident(&self) -> &#ty {
                &self.#ident
            }

            pub fn #set_ident(&mut self, value: #ty) {
                self.#ident = value;
            }
        });

        init.extend(quote! {
            #ident: {
                let size = <#ty as storage::InitStorage>::SIZE;
                if space < size {
                    space = 32;
                    slot += 1;
                }
                space -= size;

                root += alloy_primitives::U256::from(slot);
                <#ty as storage::InitStorage>::init(root, space)
            },
        });
    }

    let expanded = quote! {
        #input

        impl #impl_generics #name #ty_generics #where_clause {
            #accessors
        }

        impl #impl_generics stylus_sdk::storage::InitStorage for #name #ty_generics #where_clause {
            fn init(mut root: stylus_sdk::alloy_primitives::U256, offset: u8) -> Self {
                use stylus_sdk::{storage, alloy_primitives};
                debug_assert!(offset == 0);

                let mut space: u8 = 32;
                let mut slot: u32 = 0;
                Self {
                    #init
                }
            }
        }
    };

    TokenStream::from(expanded)
}
