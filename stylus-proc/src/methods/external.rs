// Copyright 2022-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use crate::types::{self, Purity};
use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use std::{mem, str::FromStr};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    FnArg, ImplItem, Index, ItemImpl, Lit, LitStr, Pat, PatType, Result, ReturnType, Token, Type,
};

pub fn external(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as ItemImpl);
    let mut selectors = quote!();
    let mut match_selectors = quote!();
    let mut abi = quote!();
    let mut types = vec![];
    let mut fallback = None;

    for item in input.items.iter_mut() {
        let ImplItem::Method(method) = item else {
            continue;
        };

        // see if user chose a purity or selector (TODO: use drain_filter when stable)
        let mut purity = None;
        let mut override_id = None;
        let mut override_name = None;
        for attr in mem::take(&mut method.attrs) {
            let Some(ident) = attr.path.get_ident() else {
                continue;
            };
            if let Ok(elem) = Purity::from_str(&ident.to_string()) {
                if !attr.tokens.is_empty() {
                    error!(attr.tokens, "attribute does not take parameters");
                }
                if purity.is_some() {
                    error!(attr.path, "more than one purity attribute");
                }
                purity = Some(elem);
                continue;
            }
            if *ident == "selector" {
                if override_id.is_some() || override_name.is_some() {
                    error!(attr.path, "more than one selector attribute");
                }
                let args = match syn::parse2::<SelectorArgs>(attr.tokens.clone()) {
                    Ok(args) => args,
                    Err(error) => error!(ident, "{}", error),
                };
                override_id = args.id;
                override_name = args.name;
                continue;
            }
            method.attrs.push(attr);
        }

        use Purity::*;

        // determine purity if not
        let mut args = method.sig.inputs.iter().peekable();
        let mut has_self = false;
        let needed_purity = match args.peek() {
            Some(FnArg::Receiver(recv)) => {
                has_self = true;
                recv.mutability.into()
            }
            Some(FnArg::Typed(PatType { ty, .. })) => match &**ty {
                Type::Reference(ty) => ty.mutability.into(),
                _ => Pure,
            },
            _ => Pure,
        };

        // enforce purity
        let purity = purity.unwrap_or(needed_purity);
        if purity == Pure && purity < needed_purity {
            error!(args.next(), "pure method must not access storage");
        }
        if purity == View && purity < needed_purity {
            error!(args.next(), "storage is &mut, but the method is {purity}");
        }
        if needed_purity > Pure {
            args.next(); // drop first arg
        }

        /// finds the root type for a given arg
        fn pattern_ident(pat: Pat) -> Option<Ident> {
            match pat {
                Pat::Ident(ident) => Some(ident.ident),
                Pat::Reference(pat) => pattern_ident(*pat.pat),
                _ => None,
            }
        }
        let args: Vec<_> = args
            .map(|arg| match arg {
                FnArg::Typed(t) => (pattern_ident(*t.pat.clone()), t.ty.clone()),
                _ => unreachable!(),
            })
            .collect();

        let name = &method.sig.ident;
        let sol_name = override_name.unwrap_or(name.to_string().to_case(Case::Camel));

        // deny value when method isn't payable
        let mut deny_value = quote!();
        if purity != Payable {
            let name = name.to_string();
            deny_value = quote! {
                if let Err(err) = internal::deny_value(#name) {
                    return Some(Err(err));
                }
            };
        };

        // get the needed storage
        let storage = if needed_purity == Pure {
            quote!()
        } else if has_self {
            quote! { core::borrow::BorrowMut::borrow_mut(storage), }
        } else {
            quote! { storage, }
        };

        for attr in mem::take(&mut method.attrs) {
            if !attr.path.is_ident("fallback") {
                method.attrs.push(attr);
                continue;
            }
            if fallback.is_some() {
                error!(attr.path, "more than one fallback attribute");
            }
            if !attr.tokens.is_empty() {
                error!(attr.tokens, "the fallback attribute doesn't take arguments");
            }
            if args.len() > 1 {
                error!(
                    &method.sig,
                    "`fallback` may only accept `Vec<u8>` besides `self`"
                );
            }

            if args.is_empty() {
                fallback = Some(quote! {
                    let result = Self::#name(#storage);
                    return Some(EncodableReturnType::encode(result));
                });
            } else {
                let ty = args[0].1.clone();
                if ty != parse_quote!(Vec<u8>) {
                    error!(
                        &method.sig,
                        "`fallback` may only accept `Vec<u8>` besides `self`"
                    );
                }
                fallback = Some(quote! {
                    let bytes_arg = match <#ty as AbiType>::SolType::decode(input, true) {
                        Ok(arg) => arg,
                        Err(err) => {
                            internal::failed_to_decode_arguments(err);
                            return Some(Err(vec![]));
                        }
                    };
                    let result = Self::#name(#storage bytes_arg);
                    return Some(EncodableReturnType::encode(result));
                });
            }
        }

        if fallback.is_some() {
            // We skip exporting the abi for the `fallback` function since it
            // is never present in Solidity interfaces.
            continue;
        }

        // get the solidity args
        let expand_args = (0..args.len()).map(Index::from).map(|i| quote! { args.#i });

        // calculate selector
        let constant = Ident::new(&format!("SELECTOR_{name}"), name.span());
        let arg_types: &Vec<_> = &args.iter().map(|a| &a.1).collect();

        let selector = match override_id {
            Some(id) => quote! { #id },
            None => quote! { u32::from_be_bytes(function_selector!(#sol_name #(, #arg_types)*)) },
        };
        selectors.extend(quote! {
            #[allow(non_upper_case_globals)]
            const #constant: u32 = #selector;
        });

        // match against the selector
        match_selectors.extend(quote! {
            #[allow(non_upper_case_globals)]
            #constant => {
                #deny_value
                let args = match <<( #( #arg_types, )* ) as AbiType>::SolType as SolType>::decode(input, true) {
                    Ok(args) => args,
                    Err(err) => {
                        internal::failed_to_decode_arguments(err);
                        return Some(Err(vec![]));
                    }
                };
                let result = Self::#name(#storage #(#expand_args, )* );
                Some(EncodableReturnType::encode(result))
            }
        });

        // only collect abi info if enabled
        if cfg!(not(feature = "export-abi")) {
            continue;
        }

        let sol_args = args.iter().enumerate().map(|(i, (ident, ty))| {
            let comma = (i > 0).then_some(", ").unwrap_or_default();
            let name = ident.as_ref().map(ToString::to_string).unwrap_or_default();
            quote! {
                write!(f, "{}{}{}", #comma, <#ty as AbiType>::EXPORT_ABI_ARG, underscore_if_sol(#name))?;
            }
        });
        let sol_outs = match &method.sig.output {
            ReturnType::Default => quote!(),
            ReturnType::Type(_, ty) => quote! { write_solidity_returns::<#ty>(f)?; },
        };
        let sol_purity = match purity {
            Write => "".to_string(),
            x => format!(" {x}"),
        };

        let mut comment = quote!();
        if let Some(id) = override_id {
            comment.extend(quote! {
                write!(f, "\n    // note: selector was overridden to be 0x{:x}.", #id)?;
            });
        }

        abi.extend(quote! {
            #comment
            write!(f, "\n    function {}(", #sol_name)?;
            #(#sol_args)*
            write!(f, ") external")?;
            write!(f, #sol_purity)?;
            #sol_outs
            writeln!(f, ";")?;
        });
    }

    // collect inherits
    let mut inherits = vec![];
    for attr in mem::take(&mut input.attrs) {
        if !attr.path.is_ident("inherit") {
            input.attrs.push(attr);
            continue;
        }
        let contents: InheritsAttr = match attr.parse_args() {
            Ok(contents) => contents,
            Err(err) => return proc_macro::TokenStream::from(err.to_compile_error()),
        };
        for ty in contents.types {
            inherits.push(ty);
        }
    }

    // try to match against each inherited router
    let inherit_routes = inherits.iter().map(|ty| {
        quote! {
            if let Some(result) = <#ty as Router<S>>::route(storage, selector, input) {
                return Some(result);
            }
        }
    });

    // ensure we can actually borrow the things we inherit
    let borrow_clauses = inherits.iter().map(|ty| {
        quote! {
            S: core::borrow::BorrowMut<#ty>
        }
    });

    let self_ty = &input.self_ty;
    let generic_params = &input.generics.params;
    let where_clauses = input
        .generics
        .where_clause
        .clone()
        .map(|c| c.predicates)
        .unwrap_or_default();

    // implement Router with inheritence
    let mut router = quote! {
        #input

        impl<S, #generic_params> stylus_sdk::abi::Router<S> for #self_ty
        where
            S: stylus_sdk::storage::TopLevelStorage + core::borrow::BorrowMut<Self>,
            #(#borrow_clauses,)*
            #where_clauses
        {
            // TODO: this should be configurable
            type Storage = Self;

            #[inline(always)]
            fn route(storage: &mut S, selector: u32, input: &[u8]) -> Option<stylus_sdk::ArbResult> {
                use stylus_sdk::{function_selector, alloy_sol_types::SolType};
                use stylus_sdk::abi::{internal, internal::EncodableReturnType, AbiType, Router};
                use alloc::vec;

                #[cfg(feature = "export-abi")]
                use stylus_sdk::abi::export;

                #selectors
                match selector {
                    #match_selectors
                    _ => {
                        #(#inherit_routes)*

                        #fallback
                        None
                    }
                }
            }
        }
    };

    // only collect abi info if enabled
    if cfg!(not(feature = "export-abi")) {
        return router.into();
    }

    for item in input.items.iter_mut() {
        let ImplItem::Method(method) = item else {
            continue;
        };
        if let ReturnType::Type(_, ty) = &method.sig.output {
            types.push(ty);
        }
    }

    let type_decls = quote! {
        let mut seen = HashSet::new();
        for item in [].iter() #(.chain(&<#types as InnerTypes>::inner_types()))* {
            if seen.insert(item.id) {
                writeln!(f, "\n    {}", item.name)?;
            }
        }
    };

    let name = match *self_ty.clone() {
        Type::Path(path) => path.path.segments.last().unwrap().ident.clone().to_string(),
        _ => error!(self_ty, "Can't generate ABI for unnamed type"),
    };

    let inherited_abis = inherits.iter().map(|ty| {
        quote! {
            <#ty as GenerateAbi>::fmt_abi(f)?;
            writeln!(f)?;
        }
    });

    // write the "is" clause in Solidity
    let mut is_clause = match inherits.is_empty() {
        true => quote! {},
        false => quote! { write!(f, " is ")?; },
    };
    is_clause.extend(inherits.iter().enumerate().map(|(i, ty)| {
        let comma = (i > 0).then_some(", ").unwrap_or_default();
        quote! {
            write!(f, "{}I{}", #comma, <#ty as GenerateAbi>::NAME)?;
        }
    }));

    router.extend(quote! {
        impl<#generic_params> stylus_sdk::abi::GenerateAbi for #self_ty where #where_clauses {
            const NAME: &'static str = #name;

            fn fmt_abi(f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                use stylus_sdk::abi::{AbiType, GenerateAbi};
                use stylus_sdk::abi::internal::write_solidity_returns;
                use stylus_sdk::abi::export::{underscore_if_sol, internal::InnerTypes};
                use std::collections::HashSet;
                #(#inherited_abis)*
                write!(f, "interface I{}", #name)?;
                #is_clause
                write!(f, " {{")?;
                #abi
                #type_decls
                writeln!(f, "}}")?;
                Ok(())
            }
        }
    });

    router.into()
}

struct InheritsAttr {
    types: Punctuated<Type, Token![,]>,
}

impl Parse for InheritsAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let types = Punctuated::parse_separated_nonempty(input)?;
        Ok(Self { types })
    }
}

struct SelectorArgs {
    id: Option<u32>,
    name: Option<String>,
}

impl Parse for SelectorArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut id = None;
        let mut name = None;

        let content;
        let _ = parenthesized!(content in input);
        let input = content;

        if input.is_empty() {
            error!(@input.span(), "missing id or text argument");
        }

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            let _: Token![=] = input.parse()?;

            match ident.to_string().as_str() {
                "id" => {
                    let lit: Lit = input.parse()?;
                    if id.is_some() {
                        error!(@lit, r#"only one "id" is allowed"#);
                    }
                    id = Some(match lit {
                        Lit::Int(lit) => lit.base10_parse()?,
                        Lit::Str(lit) => {
                            let name = lit.value();
                            if !name.contains('(') {
                                error!(@lit, "missing parens. Perhaps you meant name = \"{}\"?", name);
                            }
                            let hash = types::keccak(name.as_bytes());
                            u32::from_be_bytes(hash[..4].try_into().unwrap())
                        }
                        _ => error!(@lit, "expected u32 or string"),
                    });
                }
                "name" => {
                    let lit: LitStr = input.parse()?;
                    if name.is_some() {
                        error!(@lit, r#"only one "name" is allowed"#);
                    }
                    name = Some(lit.value());
                }
                _ => error!(@ident, "Unknown selector attribute"),
            }

            // allow a comma
            let _: Result<Token![,]> = input.parse();
        }

        if id.is_some() == name.is_some() {
            error!(@input.span(), r#"only one of "id" or "name" expected"#);
        }
        Ok(Self { id, name })
    }
}
