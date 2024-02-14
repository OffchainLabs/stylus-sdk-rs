// Copyright 2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use crate::storage::proc::{SolidityField, SolidityItem, SolidityFields, SolidityStruct, SolidityItems, SolidityEnum};
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use std::mem;
use syn::{parse_macro_input, punctuated::Punctuated, Index, ItemStruct, Token, Type};

mod proc;

pub fn solidity_storage(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as syn::ItemStruct);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut init = quote! {};
    let mut size = quote! {};
    let mut borrows = quote! {};

    for (field_index, field) in input.fields.iter_mut().enumerate() {
        // deny complex types
        let Type::Path(ty) = &field.ty else {
            error!(&field, "Type not supported for EVM state storage");
        };

        // implement borrows (TODO: use drain_filter when stable)
        let attrs = mem::take(&mut field.attrs);
        for attr in attrs {
            if !attr.path.is_ident("borrow") {
                field.attrs.push(attr);
                continue;
            }
            if !attr.tokens.is_empty() {
                error!(attr.tokens, "borrow attribute does not take parameters");
            }
            let ty = &field.ty;
            let accessor = match field.ident.as_ref() {
                Some(accessor) => accessor.into_token_stream(),
                None => Index::from(field_index).into_token_stream(),
            };
            borrows.extend(quote! {
                impl core::borrow::Borrow<#ty> for #name {
                    fn borrow(&self) -> &#ty {
                        &self.#accessor
                    }
                }
                impl core::borrow::BorrowMut<#ty> for #name {
                    fn borrow_mut(&mut self) -> &mut #ty {
                        &mut self.#accessor
                    }
                }
            });
        }

        let path = &ty.path.segments.last().unwrap().ident;
        let not_supported = format!("Type `{path}` not supported for EVM state storage");

        // TODO: use short-hand substition from the `storage-macro-shorthand` branch
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
            "usize" => error!(&field, "{not_supported}."), // TODO: add usize
            "isize" => error!(&field, "{not_supported}."), // TODO: add isize
            "bool" => error!(&field, "{not_supported}. Instead try `StorageBool`."),
            "f32" | "f64" => error!(&field, "{not_supported}. Consider fixed-point arithmetic."),
            _ => {}
        }

        let Some(ident) = &field.ident else {
            continue;
        };

        init.extend(quote! {
            #ident: {
                let bytes = <#ty as storage::StorageType>::SLOT_BYTES;
                let words = <#ty as storage::StorageType>::REQUIRED_SLOTS;
                if space < bytes {
                    space = 32;
                    slot += 1;
                }
                space -= bytes;

                let root = root + alloy_primitives::U256::from(slot);
                let field = <#ty as storage::StorageType>::new(root, space as u8);
                if words > 0 {
                    slot += words;
                    space = 32;
                }
                field
            },
        });

        size.extend(quote! {
            let bytes = <#ty as storage::StorageType>::SLOT_BYTES;
            let words = <#ty as storage::StorageType>::REQUIRED_SLOTS;

            if words > 0 {
                total += words;
                space = 32;
            } else {
                if space < bytes {
                    space = 32;
                    total += 1;
                }
                space -= bytes;
            }
        });
    }

    // TODO: add mechanism for struct assignment
    let expanded = quote! {
        #input

        impl #impl_generics #name #ty_generics #where_clause {
            const fn required_slots() -> usize {
                use stylus_sdk::storage;
                let mut total: usize = 0;
                let mut space: usize = 32;
                #size
                if space != 32 || total == 0 {
                    total += 1;
                }
                total
            }
        }

        impl #impl_generics stylus_sdk::storage::StorageType for #name #ty_generics #where_clause {
            type Wraps<'a> = stylus_sdk::storage::StorageGuard<'a, Self> where Self: 'a;
            type WrapsMut<'a> = stylus_sdk::storage::StorageGuardMut<'a, Self> where Self: 'a;

            // start a new word
            const SLOT_BYTES: usize = 32;
            const REQUIRED_SLOTS: usize = Self::required_slots();

            unsafe fn new(mut root: stylus_sdk::alloy_primitives::U256, offset: u8) -> Self {
                use stylus_sdk::{storage, alloy_primitives};
                debug_assert!(offset == 0);

                let mut space: usize = 32;
                let mut slot: usize = 0;
                let accessor = Self {
                    #init
                };
                accessor
            }

            fn load<'s>(self) -> Self::Wraps<'s> {
                stylus_sdk::storage::StorageGuard::new(self)
            }

            fn load_mut<'s>(self) -> Self::WrapsMut<'s> {
                stylus_sdk::storage::StorageGuardMut::new(self)
            }
        }

        #borrows
    };
    expanded.into()
}

pub fn sol_storage(input: TokenStream) -> TokenStream {
    let SolidityItems(decls) = parse_macro_input!(input as SolidityItems);
    let mut out = quote!();

    for decl in decls {
        match decl {
            SolidityItem::Struct(SolidityStruct {
                attrs,
                vis,
                name,
                generics,
                fields: SolidityFields(fields),
            }) => {
                let fields: Punctuated<_, Token![,]> = fields
                    .into_iter()
                    .map(|SolidityField { attrs, name, ty }| {
                        quote! {
                            #(#attrs)*
                            pub #name: #ty
                        }
                    })
                    .collect();

                out.extend(quote! {
                    #(#attrs)*
                    #[stylus_sdk::stylus_proc::solidity_storage]
                    #vis struct #name #generics {
                        #fields
                    }
                });

            }
            SolidityItem::Enum(SolidityEnum {
                attrs,
                vis,
                name,
                enum_name,
                variants,
            }) => {
                let variant_count = variants.len();
                let bytes = if variant_count <= 1 << 8 {
                    1usize
                } else if variant_count <= 1 << 16 {
                    2usize
                } else if variant_count <= 1 << 24 {
                    3usize
                } else if variant_count <= 1 << 32 {
                    4usize
                } else {
                    panic!("enum with too many variants")
                };

                let mut variants_to_numbers = quote!();
                let mut numbers_to_variants = quote!();
                let mut variants_quote = quote!();

                for (i, variant) in variants.iter().enumerate() {
                    let i = i as u32;
                    variants_to_numbers.extend(quote!{
                        #variant => #i,
                    });
                    numbers_to_variants.extend(quote!{
                        #i => #variant,
                    });
                    variants_quote.extend(quote!{
                        #variant,
                    })
                }


                out.extend(quote! {
                    #(#attrs)*
                    #[derive(Copy, Clone)]
                    #vis enum #enum_name {
                        #variants_quote
                    }

                    impl #enum_name {
                        fn to_u32(self) -> u32 {
                            use #enum_name::*;
                            match self {
                                #variants_to_numbers
                            }
                        }

                        fn from_u32(x: u32) -> Self {
                            use #enum_name::*;
                            match x {
                                #numbers_to_variants
                                _ => panic!(),
                            }
                        }
                    }

                    #(#attrs)*
                    #vis struct #name {
                        slot: U256,
                        offset: u8,
                        cached: ::core::cell::OnceCell<#enum_name>,
                    }

                    impl ::core::ops::Deref for #name {
                        type Target = #enum_name;

                        fn deref(&self) -> &Self::Target {
                            self.cached.get_or_init(|| {
                                let x = unsafe {
                                    ::stylus_sdk::storage::get_bytes::<#bytes>(self.slot, self.offset.into())
                                };
                                let mut arr = [0u8; 4];
                                arr[..#bytes].copy_from_slice(&x);
                                #enum_name::from_u32(u32::from_le_bytes(arr))
                            })
                        }
                    }

                    impl #name {
                        pub fn get(&self) -> #enum_name {
                            **self
                        }

                        pub fn set(&mut self, value: E) {
                            self.cached = ::core::cell::OnceCell::new();
                            let _ = self.cached.set(value);
                            let x = #enum_name::to_u32(value).to_le_bytes();
                            let mut arr = [0u8; #bytes];
                            arr[..].copy_from_slice(&x[..#bytes]);
                            unsafe {
                                ::stylus_sdk::storage::set_bytes::<#bytes>(self.slot, self.offset.into(), arr)
                            }
                        }
                    }

                    impl ::stylus_sdk::storage::StorageType for #name {
                        type Wraps<'a> = #enum_name;
                        type WrapsMut<'a> = ::stylus_sdk::storage::StorageGuardMut<'a, Self>;

                        const SLOT_BYTES: usize = #bytes;

                        unsafe fn new(slot: U256, offset: u8) -> Self {
                            Self {
                                slot,
                                offset,
                                cached: ::core::cell::OnceCell::new(),
                            }
                        }

                        fn load<'s>(self) -> Self::Wraps<'s> {
                            self.get()
                        }

                        fn load_mut<'s>(self) -> Self::WrapsMut<'s> {
                            stylus_sdk::storage::StorageGuardMut::new(self)
                        }
                    }

                    impl ::stylus_sdk::storage::Erase for #name {
                        fn erase(&mut self) {
                            self.set(#enum_name::from_u32(0));
                        }
                    }

                    impl<'a> ::stylus_sdk::storage::SimpleStorageType<'a> for #name {
                        fn set_by_wrapped(&mut self, value: Self::Wraps<'a>) {
                            self.set(value);
                        }
                    }

                    impl From<#name> for #enum_name {
                        fn from(value: #name) -> Self {
                            *value
                        }
                    }

                });
            }
        }
    }

    out.into()
}

pub fn derive_erase(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as ItemStruct);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut erase_fields = quote! {};
    for field in &mut input.fields {
        let ident = &field.ident;
        erase_fields.extend(quote! {
            self.#ident.erase();
        });
    }
    let output = quote! {
        impl #impl_generics stylus_sdk::storage::Erase for #name #ty_generics #where_clause {
            fn erase(&mut self) {
                #erase_fields
            }
        }
    };
    output.into()
}
