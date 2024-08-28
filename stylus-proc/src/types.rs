// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy_sol_types::SolType;
use proc_macro2::TokenStream;
use quote::quote;
use std::{borrow::Cow, fmt::Display, num::NonZeroU16, str::FromStr};
use syn::Token;
use syn_solidity::Type;

/// The purity of a Solidity method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Purity {
    Pure,
    View,
    Write,
    Payable,
}

impl Purity {
    /// How to reference this purity from inside a contract.
    pub fn as_tokens(&self) -> TokenStream {
        match self {
            Purity::Pure => quote! { stylus_sdk::methods::Purity::Pure },
            Purity::View => quote! { stylus_sdk::methods::Purity::View },
            Purity::Write => quote! { stylus_sdk::methods::Purity::Write },
            Purity::Payable => quote! { stylus_sdk::methods::Purity::Payable },
        }
    }
}

impl Default for Purity {
    fn default() -> Self {
        Self::Pure
    }
}

impl FromStr for Purity {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "pure" => Self::Pure,
            "view" => Self::View,
            "write" => Self::Write,
            "payable" => Self::Payable,
            _ => return Err(()),
        })
    }
}

impl Display for Purity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pure => write!(f, "pure"),
            Self::View => write!(f, "view"),
            Self::Write => write!(f, "write"),
            Self::Payable => write!(f, "payable"),
        }
    }
}

impl From<Option<Token![mut]>> for Purity {
    fn from(value: Option<Token![mut]>) -> Self {
        match value.is_some() {
            true => Self::Write,
            false => Self::View,
        }
    }
}

/// Returns the alloy path and ABI for a given Solidity type
pub fn solidity_type_info(ty: &Type) -> (Cow<'static, str>, Cow<'static, str>) {
    use alloy_sol_types::sol_data;

    macro_rules! abi {
        ($format:expr $(,$msg:expr)*) => {
            format!($format $(,$msg)*).into()
        };
    }
    macro_rules! path {
        ($format:expr $(,$msg:expr)*) => {{
            let text = format!($format $(,$msg)*);
            abi!("stylus_sdk::alloy_sol_types::sol_data::{text}")
        }};
    }
    macro_rules! simple {
        ($ty:ident) => {
            (path!(stringify!($ty)), sol_data::$ty::SOL_NAME.into())
        };
    }
    match ty {
        Type::Bool(_) => simple!(Bool),
        Type::Address(_, _) => simple!(Address),
        Type::String(_) => simple!(String),
        Type::Bytes(_) => simple!(Bytes),
        Type::FixedBytes(_, size) => (
            format!("stylus_sdk::alloy_sol_types::sol_data::FixedBytes<{size}>").into(),
            abi!("bytes[{size}]"),
        ),
        Type::Uint(_, size) => {
            let size = size.unwrap_or(NonZeroU16::new(256).unwrap());
            (path!("Uint<{size}>"), abi!("uint{size}"))
        }
        Type::Int(_, size) => {
            let size = size.unwrap_or(NonZeroU16::new(256).unwrap());
            (path!("Int<{size}>"), abi!("int{size}"))
        }
        Type::Array(ty) => {
            let (path, abi) = solidity_type_info(&ty.ty);
            match ty.size() {
                Some(size) => (path!("FixedArray<{path}, {size}>"), abi!("{abi}[{size}]")),
                None => (path!("Array<{path}>"), abi!("{abi}[]")),
            }
        }
        Type::Tuple(tup) => {
            if tup.types.is_empty() {
                ("()".into(), "()".into())
            } else if tup.types.len() == 1 {
                solidity_type_info(&tup.types[0])
            } else {
                let mut path = "(".to_string();
                let mut abi = "(".to_string();
                for (i, ty) in tup.types.iter().enumerate() {
                    if i > 0 {
                        path += ", ";
                        abi += ",";
                    }
                    let (inner_path, inner_abi) = solidity_type_info(ty);
                    path += &inner_path;
                    abi += &inner_abi;
                }
                path += ")";
                abi += ")";
                (path.into(), abi.into())
            }
        }
        _ => todo!("Solidity type {ty} is not yet implemented in sol_interface!"),
    }
}
