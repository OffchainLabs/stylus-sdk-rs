// Copyright 2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use lazy_static::lazy_static;
use proc_macro2::{Ident, Literal};
use quote::quote;
use regex::Regex;
use syn::{
    braced, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Bracket,
    Attribute, Error, Generics, Path, Result, Token, Visibility,
};

macro_rules! sdk {
    ($($msg:expr),+) => {
        format!("stylus_sdk::storage::{}", format!($($msg),+))
    };
}

pub struct SolidityStructs(pub Vec<SolidityStruct>);

impl Parse for SolidityStructs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut structs = Vec::new();
        while !input.is_empty() {
            structs.push(input.parse()?);
        }
        Ok(Self(structs))
    }
}

pub struct SolidityStruct {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub name: Ident,
    pub generics: Generics,
    pub fields: SolidityFields,
}

impl Parse for SolidityStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        // #[attrs?]
        // pub? struct name
        let attrs: Vec<Attribute> = Attribute::parse_outer(input)?;
        let vis: Visibility = input.parse()?;
        let _: Token![struct] = input.parse()?;
        let name: Ident = input.parse()?;
        let generics: Generics = input.parse()?;

        let content;
        let _ = braced!(content in input);
        let fields = content.parse()?;
        Ok(Self {
            attrs,
            vis,
            name,
            generics,
            fields,
        })
    }
}

pub struct SolidityFields(pub Punctuated<SolidityField, Token![;]>);

impl Parse for SolidityFields {
    fn parse(input: ParseStream) -> Result<Self> {
        let fields = Punctuated::parse_terminated(input)?;
        Ok(Self(fields))
    }
}

pub struct SolidityField {
    pub attrs: Vec<Attribute>,
    pub name: Ident,
    pub ty: Path,
}

impl Parse for SolidityField {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs: Vec<Attribute> = Attribute::parse_outer(input)?;
        let ty = SolidityTy::parse(input)?.0;
        let name: Ident = input.parse()?;
        Ok(SolidityField { attrs, name, ty })
    }
}

pub struct SolidityTy(Path);

impl Parse for SolidityTy {
    fn parse(input: ParseStream) -> Result<Self> {
        let start: Path = input.parse()?;
        let mut path: Path;

        if start.is_ident("mapping") {
            let content;
            let _ = parenthesized!(content in input);

            // key => value
            let key = content.parse::<PrimitiveKey>()?.0;
            let _: Token![=>] = content.parse()?;
            let value = content.parse::<SolidityTy>()?.0;

            let ty = format!(
                "{}<{}, {}>",
                sdk!("StorageMap"),
                quote!(#key),
                quote!(#value)
            );
            path = syn::parse_str(&ty)?;
        } else {
            let base: Primitive = start.try_into()?;
            path = base.0
        };

        while input.peek(Bracket) {
            let content;
            let _ = bracketed!(content in input);
            if let Ok(bracket_contents) = content.parse::<Literal>() {
                let size = bracket_contents.to_string().parse::<usize>().map_err(|_| {
                    Error::new_spanned(&bracket_contents, "Array size must be a positive integer")
                })?;
                let outer = sdk!("StorageArray");
                let inner = quote! { #path };
                path = syn::parse_str(&format!("{outer}<{inner}, {size}>"))?;
                return Ok(SolidityTy(path));
            };

            let outer = sdk!("StorageVec");
            let inner = quote! { #path };
            path = syn::parse_str(&format!("{outer}<{inner}>"))?;
        }

        Ok(SolidityTy(path))
    }
}

pub struct Primitive(Path);

lazy_static! {
    static ref UINT_REGEX: Regex = Regex::new(r"^uint(\d+)$").unwrap();
    static ref INT_REGEX: Regex = Regex::new(r"^int(\d+)$").unwrap();
    static ref BYTES_REGEX: Regex = Regex::new(r"^bytes(\d+)$").unwrap();
    static ref LOWER_REGEX: Regex = Regex::new(r"^[0-9a-z]+$").unwrap();
}

impl Parse for Primitive {
    fn parse(input: ParseStream) -> Result<Self> {
        let path: Path = input.parse()?;
        path.try_into()
    }
}

impl TryFrom<Path> for Primitive {
    type Error = Error;

    fn try_from(path: Path) -> std::result::Result<Self, Self::Error> {
        let Some(ident) = path.get_ident() else {
            return Ok(Self(path));
        };
        let name = &ident.to_string();

        macro_rules! ty {
            ($($msg:expr),+) => {{
                let path = sdk!($($msg),+);
                Ok(Self(syn::parse_str(&path)?))
            }};
        }
        macro_rules! error {
            ($msg:expr) => {
                Err(Error::new_spanned(&ident, $msg))
            };
        }

        if let Some(caps) = UINT_REGEX.captures(name) {
            let bits: usize = caps[1].parse().unwrap();
            let limbs = (63 + bits) / 64;
            if bits > 256 {
                return error!("Type not supported: too many bits");
            }
            return ty!("StorageUint<{}, {}>", bits, limbs);
        }

        if let Some(caps) = INT_REGEX.captures(name) {
            let bits: usize = caps[1].parse().unwrap();
            let limbs = (63 + bits) / 64;
            if bits > 256 {
                return error!("Type not supported: too many bits");
            }
            return ty!("StorageSigned<{}, {}>", bits, limbs);
        }

        if let Some(caps) = BYTES_REGEX.captures(name) {
            let bytes: usize = caps[1].parse().unwrap();
            if bytes > 32 {
                return error!("Type not supported: too many bytes");
            }
            return ty!("StorageFixedBytes<{}>", bytes);
        }

        let ty = match name.as_str() {
            "address" => "StorageAddress",
            "bool" => "StorageBool",
            "bytes" => "StorageBytes",
            "int" => "StorageI256",
            "string" => "StorageString",
            "uint" => "StorageU256",
            x => match LOWER_REGEX.is_match(x) {
                true => return Err(Error::new_spanned(ident, "Type not supported")),
                false => return Ok(Self(syn::parse_str(x)?)),
            },
        };

        ty!("{ty}")
    }
}

pub struct PrimitiveKey(Path);

impl Parse for PrimitiveKey {
    fn parse(input: ParseStream) -> Result<Self> {
        let path: Path = input.parse()?;
        path.try_into()
    }
}

impl TryFrom<Path> for PrimitiveKey {
    type Error = Error;

    fn try_from(path: Path) -> std::result::Result<Self, Self::Error> {
        let Some(ident) = path.get_ident() else {
            return Ok(Self(path));
        };
        let name = &ident.to_string();

        macro_rules! ty {
            ($($msg:expr),+) => {{
                let path = format!($($msg),+);
                let path = format!("stylus_sdk::alloy_primitives::{path}");
                Ok(Self(syn::parse_str(&path)?))
            }};
        }
        macro_rules! error {
            ($msg:expr) => {
                Err(Error::new_spanned(&ident, $msg))
            };
        }

        if let Some(caps) = UINT_REGEX.captures(name) {
            let bits: usize = caps[1].parse().unwrap();
            let limbs = (63 + bits) / 64;
            if bits > 256 {
                return error!("Type not supported: too many bits");
            }
            return ty!("Uint<{}, {}>", bits, limbs);
        }

        if let Some(caps) = INT_REGEX.captures(name) {
            let bits: usize = caps[1].parse().unwrap();
            let limbs = (63 + bits) / 64;
            if bits > 256 {
                return error!("Type not supported: too many bits");
            }
            return ty!("Signed<{}, {}>", bits, limbs);
        }

        if let Some(caps) = BYTES_REGEX.captures(name) {
            let bytes: usize = caps[1].parse().unwrap();
            if bytes > 32 {
                return error!("Type not supported: too many bytes");
            }
            return ty!("FixedBytes<{}>", bytes);
        }

        let ty = match name.as_str() {
            "address" => "Address",
            "bool" => "U8", // TODO: ask alloy to add a Bool type
            "int" => "I256",
            "uint" => "U256",
            "bytes" => return Ok(Self(syn::parse_str("Vec<u8>")?)),
            "string" => return Ok(Self(syn::parse_str("String")?)),
            x => match LOWER_REGEX.is_match(x) {
                true => return Err(Error::new_spanned(ident, "Type not supported")),
                false => return Ok(Self(syn::parse_str(x)?)),
            },
        };

        ty!("{ty}")
    }
}
