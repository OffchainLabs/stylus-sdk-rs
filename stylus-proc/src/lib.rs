// Copyright 2022-2023, Offchain Labs, Inc.
// For license information, see https://github.com/OffchainLabs/nitro/blob/master/LICENSE

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    token::Bracket,
    ItemStruct, Path, Result, Token, Type,
};

#[proc_macro_attribute]
pub fn solidity_storage(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as ItemStruct);

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    macro_rules! error {
        ($tokens:expr, $($msg:expr),+) => {{
            let error = syn::Error::new_spanned($tokens, format!($($msg),+));
            return error.to_compile_error().into();
        }};
    }

    let mut init = quote! {};

    if input.fields.is_empty() {
        error!(input, "Empty structs are not allowed in Solidity");
    }

    for field in &mut input.fields {
        // deny complex types
        let Type::Path(ty) = &mut field.ty else {
            error!(&field, "Type not supported for EVM state storage");
        };

        let path = &ty.path.segments.last().unwrap().ident;
        let not_supported = format!("Type `{path}` not supported for EVM state storage");

        match path.to_string().as_str() {
            x @ ("u8" | "u16" | "u32" | "u64" | "u128" | "i8" | "i16" | "i32" | "i64" | "i128"
            | "U8" | "U16" | "U32" | "U64" | "U128" | "I8" | "I16" | "I32" | "I64"
            | "I128") => {
                error!(
                    &field,
                    "{not_supported}. Instead try `Storage{}`.",
                    x.to_uppercase()
                );
            }
            "usize" => error!(&field, "{not_supported}. Instead try `StorageUsize`."),
            "isize" => error!(&field, "{not_supported}. Instead try `StorageIsize`."),
            "bool" => error!(&field, "{not_supported}. Instead try `StorageBool`."),
            _ => {}
        }

        let Some(ident) = &field.ident else {
            continue;
        };

        init.extend(quote! {
            #ident: {
                let size = <#ty as storage::StorageType>::SIZE;
                if space < size {
                    space = 32;
                    slot += 1;
                }
                space -= size;

                root += alloy_primitives::U256::from(slot);
                <#ty as storage::StorageType>::new(root, space)
            },
        });
    }

    let expanded = quote! {
        #input

        impl #impl_generics stylus_sdk::storage::StorageType for #name #ty_generics #where_clause {
            type Wraps<'a> = stylus_sdk::storage::StorageGuard<'a, #name> where Self: 'a;
            type WrapsMut<'a> = stylus_sdk::storage::StorageGuardMut<'a, #name> where Self: 'a;

            unsafe fn new(mut root: stylus_sdk::alloy_primitives::U256, offset: u8) -> Self {
                use stylus_sdk::{storage, alloy_primitives};
                debug_assert!(offset == 0);

                let mut space: u8 = 32;
                let mut slot: u32 = 0;
                Self {
                    #init
                }
            }

            fn load<'s>(self) -> Self::Wraps<'s> {
                stylus_sdk::storage::StorageGuard::new(self)
            }

            fn load_mut<'s>(self) -> Self::WrapsMut<'s> {
                stylus_sdk::storage::StorageGuardMut::new(self)
            }
        }
    };

    TokenStream::from(expanded)
}

struct SolidityField {
    pub name: Ident,
    pub ty: Path,
}

impl Parse for SolidityField {
    fn parse(input: ParseStream) -> Result<Self> {
        let ty: Ident = input.parse()?;
        let sdk = |x| format!("stylus_sdk::storage::{x}");
        
        let base = match ty.to_string().as_str() {
            "bool" => sdk("StorageBool"),
            "address" => sdk("StorageAddress"),
            name => match name.chars().all(|x| x.is_ascii_lowercase()) {
                false => name.to_string(),
                true => return Err(input.error("Unsupported type")),
            }
        };
        
        let mut ty = syn::parse_str(&base)?;

        while input.peek(Bracket) {
            let _content;
            let _brackets = bracketed!(_content in input); // TODO: fixed arrays
            let outer = sdk("StorageVec");
            let inner = quote! { #ty };
            ty = syn::parse_str(&format!("{outer}<{inner}>"))?;
        }

        let name: Ident = input.parse()?;

        Ok(SolidityField { name, ty })
    }
}

struct SolidityFields(Punctuated<SolidityField, Token![;]>);

impl Parse for SolidityFields {
    fn parse(input: ParseStream) -> Result<Self> {
        let fields = Punctuated::parse_terminated(input)?;
        Ok(Self(fields))
    }
}

#[proc_macro]
pub fn sol_storage(input: TokenStream) -> TokenStream {
    let SolidityFields(input) = parse_macro_input!(input as SolidityFields);

    let fields: Punctuated<_, Token![,]> = input
        .into_iter()
        .map(|SolidityField { name, ty }| quote! { pub #name: #ty })
        .collect();

    let item: ItemStruct = parse_quote! {
        #[stylus_sdk::stylus_proc::solidity_storage]
        pub struct Contract {
            #fields
        }
    };

    TokenStream::from(quote! { #item })
}
