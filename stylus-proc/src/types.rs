// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md
use std::{fmt::Display, num::NonZeroU16, str::FromStr};

use quote::quote;
use syn::{parse_quote, Token};

use crate::imports::alloy_sol_types::sol_data;

/// The purity of a Solidity method
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Purity {
    #[default]
    Pure,
    View,
    Write,
    Payable,
}

impl Purity {
    /// Infer the purity of the function by inspecting the first argument. Also returns whether the
    /// function has a self parameter.
    pub fn infer(sig: &syn::Signature) -> (Self, bool) {
        match sig.inputs.first() {
            Some(syn::FnArg::Receiver(recv)) => (recv.mutability.into(), true),
            Some(syn::FnArg::Typed(syn::PatType { ty, .. })) => match &**ty {
                syn::Type::Reference(ty) => (ty.mutability.into(), false),
                _ => (Self::Pure, false),
            },
            _ => (Self::Pure, false),
        }
    }

    pub fn get_context_and_call(&self) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
        match self {
            Purity::Pure | Purity::View => (
                quote!(stylus_sdk::stylus_core::calls::StaticCallContext),
                quote!(stylus_sdk::call::static_call),
            ),
            Purity::Write => (
                quote!(stylus_sdk::stylus_core::calls::NonPayableCallContext),
                quote!(stylus_sdk::call::call),
            ),
            Purity::Payable => (
                quote!(stylus_sdk::stylus_core::calls::MutatingCallContext),
                quote!(stylus_sdk::call::call),
            ),
        }
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

/// Alloy type and ABI for a Solidity type
#[derive(Debug)]
pub struct SolidityTypeInfo {
    pub alloy_type: syn::Type,
    pub sol_type: syn_solidity::Type,
}

impl SolidityTypeInfo {
    fn new(alloy_type: syn::Type, sol_type: syn_solidity::Type) -> Self {
        Self {
            alloy_type,
            sol_type,
        }
    }
}

/// Get type info from given Solidity type
impl From<&syn_solidity::Type> for SolidityTypeInfo {
    fn from(sol_type: &syn_solidity::Type) -> Self {
        use syn_solidity::Type;

        let alloy_type = match sol_type {
            Type::Bool(_) => sol_data::join("Bool"),
            Type::Address(_, _) => sol_data::join("Address"),
            Type::String(_) => sol_data::join("String"),
            Type::Bytes(_) => sol_data::join("Bytes"),
            Type::FixedBytes(_, size) => sol_data::join(&format!("FixedBytes<{size}>")),
            Type::Uint(_, size) => {
                let size = size.unwrap_or(NonZeroU16::new(256).unwrap());
                sol_data::join(&format!("Uint<{size}>"))
            }
            Type::Int(_, size) => {
                let size = size.unwrap_or(NonZeroU16::new(256).unwrap());
                sol_data::join(&format!("Int<{size}>"))
            }
            Type::Array(array) => {
                let Self { alloy_type, .. } = Self::from(&*array.ty);
                match array.size() {
                    Some(size) => {
                        parse_quote!(stylus_sdk::alloy_sol_types::sol_data::FixedArray<#alloy_type, #size>)
                    }
                    None => parse_quote!(stylus_sdk::alloy_sol_types::sol_data::Array<#alloy_type>),
                }
            }
            Type::Tuple(tup) => {
                if tup.types.is_empty() {
                    parse_quote! { () }
                } else if tup.types.len() == 1 {
                    return Self::from(&tup.types[0]);
                } else {
                    let type_info = tup.types.iter().map(Self::from);
                    let alloy_types = type_info.clone().map(|info| info.alloy_type);
                    parse_quote! {
                        (#(#alloy_types,)*)
                    }
                }
            }
            Type::Custom(path) => {
                let path = syn::Path {
                    leading_colon: None,
                    segments: path.iter().cloned().map(syn::PathSegment::from).collect(),
                };
                syn::TypePath { qself: None, path }.into()
            }
            _ => todo!("Solidity type {sol_type} is not yet implemented in sol_interface!"),
        };
        Self::new(alloy_type, sol_type.clone())
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::SolidityTypeInfo;

    macro_rules! sol_type_test {
        ($sol:ident, $alloy:ty) => {
            sol_type_test!($sol, stringify!($sol), @parse_quote!($alloy));
        };
        ($name:ident, $sol:expr, $alloy:ty) => {
            sol_type_test!($name, $sol, @parse_quote!($alloy));
        };
        ($name:ident, $sol:expr, @$alloy:expr) => {
            paste::paste! {
                #[test]
                fn [<test_sol_ $name>]() {
                    let sol_type = syn::parse_str($sol).unwrap();
                    let info = SolidityTypeInfo::from(&sol_type);
                    assert_eq!(info.sol_type, sol_type);
                    assert_eq!(info.sol_type.to_string(), $sol);
                    assert_eq!(
                        info.alloy_type,
                        $alloy,
                    );
                }
            }
        };
    }

    sol_type_test!(bool, stylus_sdk::alloy_sol_types::sol_data::Bool);
    sol_type_test!(address, stylus_sdk::alloy_sol_types::sol_data::Address);
    sol_type_test!(string, stylus_sdk::alloy_sol_types::sol_data::String);
    sol_type_test!(bytes, stylus_sdk::alloy_sol_types::sol_data::Bytes);
    sol_type_test!(
        fixed_bytes,
        "bytes10",
        stylus_sdk::alloy_sol_types::sol_data::FixedBytes<10>
    );
    sol_type_test!(uint160, stylus_sdk::alloy_sol_types::sol_data::Uint<160>);
    sol_type_test!(int32, stylus_sdk::alloy_sol_types::sol_data::Int<32>);
    #[rustfmt::skip]
    sol_type_test!(
        array,
        "int256[]",
        stylus_sdk::alloy_sol_types::sol_data::Array<
            stylus_sdk::alloy_sol_types::sol_data::Int<256>
        >
    );
    #[rustfmt::skip]
    sol_type_test!(
        fixed_array,
        "int256[100]",
        stylus_sdk::alloy_sol_types::sol_data::FixedArray<
            stylus_sdk::alloy_sol_types::sol_data::Int<256>,
            100usize
        >
    );
    sol_type_test!(
        tuple,
        "(uint256,bytes,string)",
        @parse_quote! {(
            stylus_sdk::alloy_sol_types::sol_data::Uint<256>,
            stylus_sdk::alloy_sol_types::sol_data::Bytes,
            stylus_sdk::alloy_sol_types::sol_data::String,
        )}
    );
    sol_type_test!(custom_path, "foo.bar.baz", foo::bar::baz);
}
