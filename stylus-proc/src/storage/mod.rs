// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use crate::storage::proc::{SolidityField, SolidityFields, SolidityStruct, SolidityStructs};
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use std::mem;
use syn::{parse_macro_input, punctuated::Punctuated, Index, ItemStruct, Token, Type};

mod proc;

pub fn solidity_storage(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as ItemStruct);

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut init = quote! {};
    let mut size = quote! {};
    let mut borrows = quote! {};
    let mut inner_storage_accessors = vec![];

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
            
            inner_storage_accessors.push(accessor.clone());
            
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
    
    let inner_storage_calls = inner_storage_accessors.into_iter().map(|accessor|{
        quote! {
            .or(self.#accessor.try_get_storage())
        }
    });
    
    let storage_impl = quote! {
        #[allow(clippy::transmute_ptr_to_ptr)]
        unsafe impl #impl_generics stylus_sdk::storage::InnerStorage for #name #ty_generics #where_clause {
            unsafe fn try_get_storage<S: 'static>(&mut self) -> Option<&mut S> {
                use core::any::TypeId;
                use stylus_sdk::storage::InnerStorage;
                if TypeId::of::<S>() == TypeId::of::<Self>() {
                    Some(unsafe { core::mem::transmute::<_, _>(self) })
                } else {
                    None #(#inner_storage_calls)*
                }
            }
        }
    };

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
        
        #storage_impl
    };
    expanded.into()
}

pub fn sol_storage(input: TokenStream) -> TokenStream {
    let SolidityStructs(decls) = parse_macro_input!(input as SolidityStructs);
    let mut out = quote!();

    for decl in decls {
        let SolidityStruct {
            attrs,
            vis,
            name,
            generics,
            fields: SolidityFields(fields),
        } = decl;

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
    quote! {
        impl #impl_generics stylus_sdk::storage::Erase for #name #ty_generics #where_clause {
            fn erase(&mut self) {
                #erase_fields
            }
        }
    }
    .into()
}
