// Copyright 2022-2023, Offchain Labs, Inc.
// For license information, see https://github.com/OffchainLabs/nitro/blob/master/LICENSE

use case::CaseExt;
use keccak_const::Keccak256;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span as Span2, TokenStream as TokenStream2, TokenTree as TokenTree2};
use quote::{format_ident, quote, ToTokens};
use storage::{SolidityField, SolidityFields, SolidityStruct, SolidityStructs};
use syn::{
    parse_macro_input, punctuated::Punctuated, ItemStruct, Path, PathArguments, PathSegment,
    ReturnType, Token, Type, TypeTuple,
};

// use handler::calldata_type_template;
use handler::{
    calldata_sig_name_template, calldata_type_template, generated_handler_name_template,
    returndata_type_template,
};
use router::*;

mod handler;
mod router;
mod storage;
mod ty;

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

fn extract_type_path(ty: &syn::Type) -> Option<&Path> {
    match *ty {
        syn::Type::Path(ref typepath) if typepath.qself.is_none() => Some(&typepath.path),
        _ => None,
    }
}

fn extract_single_generic_const_abi_string(arguments: &PathArguments) -> String {
    let mut generic_abi_string: String = "".to_string();

    if let PathArguments::AngleBracketed(ref ab) = *arguments {
        let arg = ab.args.first().unwrap();
        if let syn::GenericArgument::Const(syn::Expr::Lit(ref expr_lit)) = *arg {
            if let syn::Lit::Int(ref int_lit) = expr_lit.lit {
                generic_abi_string = int_lit.base10_digits().to_string();
            }
        }
    }

    generic_abi_string
}

fn extract_array_generics(arguments: &PathArguments) -> String {
    let mut generic_abi_strings = Vec::<String>::new();

    if let PathArguments::AngleBracketed(ref ab) = *arguments {
        let _ = &ab.args.iter().for_each(|gen_arg| {
            if let syn::GenericArgument::Type(ref ty) = *gen_arg {
                if let Some(type_path) = extract_type_path(ty) {
                    if let Some(last_segment) = type_path.segments.iter().last() {
                        let abi_string = sol_type_path_segment_to_abi_string(last_segment);
                        generic_abi_strings.push(abi_string.to_string());
                    }
                }
            }
        });
    }

    generic_abi_strings.push("[]".to_string());
    generic_abi_strings.join("")
}

fn extract_uint_generic(arguments: &PathArguments) -> String {
    let single_const_generic = extract_single_generic_const_abi_string(arguments);

    format!("uint{}", single_const_generic)
}

fn extract_int_generic(arguments: &PathArguments) -> String {
    let single_const_generic = extract_single_generic_const_abi_string(arguments);

    format!("int{}", single_const_generic)
}

fn sol_type_path_segment_to_abi_string(path_segment: &PathSegment) -> String {
    let PathSegment { ident, arguments } = path_segment;

    match ident.to_string().as_ref() {
        "Address" => "address".to_string(),
        "Bool" => "bool".to_string(),
        "Bytes" => "bytes".to_string(),
        "Int" => extract_int_generic(arguments),
        "Uint" => extract_uint_generic(arguments),
        "Array" => extract_array_generics(arguments),
        "String" => "string".to_string(),
        // ByteCount<N>
        // FixedArray format!("{}[{}]", T::sol_type_name(), N).into() FixedArray<T, N>
        // tuple
        _ => {
            println!("............unknown type: {:?}", ident);
            println!("arguments: {:?}", arguments);
            "unknown".to_string()
        }
    }
}

fn gen_selector(prefix: String, abi_params: String) -> u32 {
    let abi_signature = format!("{}{}", prefix.as_str(), abi_params.as_str());
    let bytes: [u8; 32] = Keccak256::new().update(abi_signature.as_bytes()).finalize();
    let selector_bytes: [u8; 4] = bytes[..4].try_into().unwrap();
    u32::from_be_bytes(selector_bytes)
}

#[proc_macro_attribute]
pub fn handler(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    if let syn::Item::Fn(function_item) = syn::parse(item.clone()).unwrap() {
        let user_handler_name = function_item.sig.ident.clone();
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

        let param_type_path_segments: Vec<&PathSegment> = param_types
            .iter()
            .filter_map(|ty| {
                if let Some(type_path) = extract_type_path(ty) {
                    if let Some(last_segment) = type_path.segments.last() {
                        return Some(last_segment);
                    }
                }
                None
            })
            .collect();

        let sol_sigs: Vec<String> = param_type_path_segments
            .iter()
            .map(|path_segment| sol_type_path_segment_to_abi_string(path_segment))
            .collect();

        let param_idents = params.iter().filter_map(|param| {
            if let syn::FnArg::Typed(pat_type) = param {
                if let syn::Pat::Ident(pat_ident) = *pat_type.pat.clone() {
                    return Some(pat_ident.ident);
                }
            }
            None
        });

        let generated_handler_name =
            format_ident!(generated_handler_name_template!(), user_handler_name);
        let calldata_type_alias = format_ident!(calldata_type_template!(), user_handler_name);
        let returndata_type_alias = format_ident!(returndata_type_template!(), user_handler_name);
        let calldata_sig_name = format_ident!(
            calldata_sig_name_template!(),
            user_handler_name.to_string().to_uppercase()
        );

        let formatted_calldata_sig = format!("({})", sol_sigs.join(","));
        let calldata_param_sig_ident = formatted_calldata_sig.as_str();

        let sol_type_calldata_sig = quote! {
          (#(#param_types ,)*)
        };

        let rust_type_calldata_sig = quote! {
          (#(#param_idents: <::stylus_sdk::alloy_sol_types::#param_types as ::stylus_sdk::alloy_sol_types::SolType>::RustType,)*)
        };

        let (sol_type_returndata_sig, rust_type_returndata_sig) = match output {
            // Default return is when no return value is specified
            ReturnType::Default => (quote! { () }, quote! {}),
            ReturnType::Type(_, box_type) => {
                let return_type = *box_type;
                (
                    quote! { #return_type },
                    quote! { -> <#return_type as ::stylus_sdk::alloy_sol_types::SolType>::RustType},
                )
            }
        };

        let user_handler_snake = user_handler_name.to_string().to_snake();
        let user_handler_camel = user_handler_name.to_string().to_camel_lowercase();
        let snake = user_handler_snake.as_str();
        let camel = user_handler_camel.as_str();

        let user_handler_snake_selector = gen_selector(
            user_handler_snake.to_owned(),
            calldata_param_sig_ident.to_string(),
        );
        let user_handler_camel_selector = gen_selector(
            user_handler_camel.to_owned(),
            calldata_param_sig_ident.to_string(),
        );

        println!("{} - {}", calldata_sig_name, calldata_param_sig_ident);
        println!(
            "prefixes: {} {}; selectors: {:x} {:x}",
            snake, camel, user_handler_snake_selector, user_handler_camel_selector
        );

        let gen = quote! {
            #[allow(non_camel_case_types)]
            type #calldata_type_alias = #sol_type_calldata_sig;

            #[allow(non_camel_case_types)]
            type #returndata_type_alias = #sol_type_returndata_sig;

            #[allow(non_snake_case)]
            fn #user_handler_name #generics #rust_type_calldata_sig #rust_type_returndata_sig {
                #block
            }

            #[allow(non_snake_case)]
            pub fn #generated_handler_name(input: Vec<u8>) -> Result<Vec<u8>, Vec<u8>> {
              use stylus_sdk::router::handler::*;
              use stylus_sdk::alloy_sol_types::{sol_data, SolType};

              let args = <#calldata_type_alias as ::stylus_sdk::alloy_sol_types::SolType>::decode(&input, true).unwrap();
              let result = (#user_handler_name).apply(args);
              let encoded_response = <#returndata_type_alias as ::stylus_sdk::alloy_sol_types::SolType>::encode(&result);

              Ok(encoded_response)
            }

            struct #calldata_sig_name;

            impl #calldata_sig_name {
              const PARAM_SIGNATURE: &'static str = #calldata_param_sig_ident;
              const CAMEL_SELECTOR: u32 = #user_handler_camel_selector;
              const SNAKE_SELECTOR: u32 = #user_handler_snake_selector;
            }
        };

        gen.into()
    } else {
        item
    }
}

#[proc_macro]
pub fn router(input: TokenStream) -> TokenStream {
    let parsed_router = parse_macro_input!(input as RouterParser);
    parsed_router.expand().into()
}
