// Copyright 2022-2023, Offchain Labs, Inc.
// For license information, see https://github.com/OffchainLabs/nitro/blob/master/LICENSE

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span as Span2, TokenStream as TokenStream2, TokenTree as TokenTree2};
use quote::{format_ident, quote, ToTokens};
use router::*;
use storage::{SolidityField, SolidityFields, SolidityStruct, SolidityStructs};
use syn::{
    parse_macro_input, punctuated::Punctuated, ItemStruct, ReturnType, Token, Type, TypeTuple,
};

mod router;
mod storage;

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
    let mut size = quote! {};

    if input.fields.is_empty() {
        error!(name, "Empty structs are not allowed in Solidity");
    }

    for field in &mut input.fields {
        // deny complex types
        let Type::Path(ty) = &mut field.ty else {
            error!(&field, "Type not supported for EVM state storage");
        };

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
                if space != 32 {
                    total += 1;
                }
                total
            }
        }

        impl #impl_generics stylus_sdk::storage::StorageType for #name #ty_generics #where_clause {
            type Wraps<'a> = stylus_sdk::storage::StorageGuard<'a, #name> where Self: 'a;
            type WrapsMut<'a> = stylus_sdk::storage::StorageGuardMut<'a, #name> where Self: 'a;

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
    };

    TokenStream::from(expanded)
}

#[proc_macro]
pub fn sol_storage(input: TokenStream) -> TokenStream {
    let SolidityStructs(decls) = parse_macro_input!(input as SolidityStructs);
    let mut out = quote!();

    for decl in decls {
        let SolidityStruct {
            vis,
            name,
            fields: SolidityFields(fields),
        } = decl;

        let fields: Punctuated<_, Token![,]> = fields
            .into_iter()
            .map(|SolidityField { name, ty }| quote! { pub #name: #ty })
            .collect();

        out.extend(quote! {
            #[stylus_sdk::stylus_proc::solidity_storage]
            #vis struct #name {
                #fields
            }
        });
    }

    out.into()
}

// fn transform_params(params: Punctuated<syn::FnArg, syn::token::Comma>) -> Expr {
//     // 1. Filter the params, so that only typed arguments remain
//     // 2. Extract the ident (in case the pattern type is ident)
//     let idents = params.iter().filter_map(|param| {
//         if let syn::FnArg::Typed(pat_type) = param {
//             if let syn::Pat::Ident(pat_ident) = *pat_type.pat.clone() {
//                 return Some(pat_ident.ident);
//             }
//         }
//         None
//     });

//     // Add all idents to a Punctuated => param1, param2, ...
//     let mut punctuated: Punctuated<syn::Ident, Comma> = Punctuated::new();
//     idents.for_each(|ident| punctuated.push(ident));

//     // Generate expression from Punctuated (and wrap with parentheses)
//     let transformed_params = parse_quote!((#punctuated));
//     transformed_params
// };
//

/**
 * Change handler macro to generate a duplicate function
 * 1. const ABI_SIG = "(address)"
 *
 * // Generated by Handler proc macro for balance_of_handler
 * type __balance_of_handler__Calldata = (...sol_data types of handler);
 * type __balance_of_handler__Returndata = (...sol_data types of return);
 *
 *
 * // Router unfurls to:
 *
 * match selector {
 *    
 * }
 *
 *
 * router! {
 *    "balance_of" => balance_of_handler,
 * }
 */

#[proc_macro_attribute]
pub fn handler(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    if let syn::Item::Fn(function_item) = syn::parse(item.clone()).unwrap() {
        let name = function_item.sig.ident.clone();
        let params = function_item.sig.inputs.clone();
        let generics = function_item.sig.generics.clone();
        let block = function_item.block.clone();
        let output = function_item.sig.output.clone();

        let param_types: Vec<Type> = params
            .iter()
            .filter_map(|param| {
                if let syn::FnArg::Typed(pat_type) = param {
                    return Some(*pat_type.ty.clone());
                }
                None
            })
            .collect();

        let param_idents = params.iter().filter_map(|param| {
            if let syn::FnArg::Typed(pat_type) = param {
                if let syn::Pat::Ident(pat_ident) = *pat_type.pat.clone() {
                    return Some(pat_ident.ident);
                }
            }
            None
        });

        let calldata_ident = format_ident!("__{}__Calldata", name);
        let handler_ident = format_ident!("__{}__Handler", name);
        let returndata_ident = format_ident!("__{}__Returndata", name);

        let sol_type_calldata_sig = quote! {
          (#(#param_types ,)*)
        };

        let rust_type_calldata_sig = quote! {
          (#(#param_idents: <::stylus_sdk::alloy_sol_types::#param_types as ::stylus_sdk::alloy_sol_types::SolType>::RustType,)*)
        };

        let (sol_type_returndata_sig, rust_type_returndata_sig) = match output {
            ReturnType::Default => (quote! { () }, quote! {}),
            ReturnType::Type(_, box_type) => {
                let return_type = *box_type;
                (
                    quote! { #return_type },
                    quote! { -> <#return_type as ::stylus_sdk::alloy_sol_types::SolType>::RustType},
                )
            }
        };

        let gen = quote! {
            #[allow(non_camel_case_types)]
            type #calldata_ident = #sol_type_calldata_sig;

            #[allow(non_camel_case_types)]
            type #returndata_ident = #sol_type_returndata_sig;

            #[allow(non_snake_case)]
            fn #name #generics #rust_type_calldata_sig #rust_type_returndata_sig {
                #block
            }

            #[allow(non_snake_case)]
            pub fn #handler_ident(input: Vec<u8>) -> Result<Vec<u8>, Vec<u8>> {
              use stylus_sdk::handler::*;
              use stylus_sdk::alloy_sol_types::{sol_data, SolType};

              let args = <#calldata_ident as ::stylus_sdk::alloy_sol_types::SolType>::decode(&input, true).unwrap();
              let result = (#name).apply(args);
              let encoded_response = <#returndata_ident as ::stylus_sdk::alloy_sol_types::SolType>::encode(&result);

              Ok(encoded_response)
            }
        };

        gen.into()
    } else {
        item
    }
}

#[proc_macro]
pub fn router(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Router);

    println!("router: {:?}", input);

    input.expand().into()
}
