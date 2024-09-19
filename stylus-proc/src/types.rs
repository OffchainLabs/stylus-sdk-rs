// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::{borrow::Cow, fmt::Display, num::NonZeroU16, str::FromStr};

use alloy_sol_types::SolType;
use syn::{parse_quote, Token};

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
    pub fn as_path(&self) -> syn::Path {
        match self {
            Purity::Pure => parse_quote!(stylus_sdk::methods::Purity::Pure),
            Purity::View => parse_quote!(stylus_sdk::methods::Purity::View),
            Purity::Write => parse_quote!(stylus_sdk::methods::Purity::Write),
            Purity::Payable => parse_quote!(stylus_sdk::methods::Purity::Payable),
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

/// Alloy type and ABI for a Solidity type
#[derive(Debug)]
pub struct SolidityTypeInfo {
    pub alloy_type: syn::Type,
    pub sol_name: Cow<'static, str>,
}

impl SolidityTypeInfo {
    fn new(alloy_type: syn::Type, sol_name: Cow<'static, str>) -> Self {
        Self {
            alloy_type,
            sol_name,
        }
    }
}

/// Get type info from given Solidity type
impl From<&syn_solidity::Type> for SolidityTypeInfo {
    fn from(ty: &syn_solidity::Type) -> Self {
        use syn_solidity::Type;

        macro_rules! parse {
            ($format:expr $(,$msg:expr)*) => {
                syn::parse_str(&format!($format $(,$msg)*)).unwrap()
            };
        }

        macro_rules! sol_data {
            ($format:expr $(,$msg:expr)*) => {{
                let text = format!($format $(,$msg)*);
                parse!("stylus_sdk::alloy_sol_types::sol_data::{text}")
            }};
        }

        macro_rules! simple {
            ($ty:ident) => {
                Self::new(
                    sol_data!(stringify!($ty)),
                    alloy_sol_types::sol_data::$ty::SOL_NAME.into(),
                )
            };
        }

        match ty {
            Type::Bool(_) => simple!(Bool),
            Type::Address(_, _) => simple!(Address),
            Type::String(_) => simple!(String),
            Type::Bytes(_) => simple!(Bytes),
            Type::FixedBytes(_, size) => Self {
                alloy_type: sol_data!("FixedBytes<{size}>"),
                sol_name: format!("bytes{size}").into(),
            },
            Type::Uint(_, size) => {
                let size = size.unwrap_or(NonZeroU16::new(256).unwrap());
                Self::new(sol_data!("Uint<{size}>"), format!("uint{size}").into())
            }
            Type::Int(_, size) => {
                let size = size.unwrap_or(NonZeroU16::new(256).unwrap());
                Self::new(sol_data!("Int<{size}>"), format!("int{size}").into())
            }
            Type::Array(ty) => {
                let Self {
                    alloy_type,
                    sol_name,
                } = Self::from(&*ty.ty);
                match ty.size() {
                    Some(size) => Self::new(
                        parse_quote!(stylus_sdk::alloy_sol_types::sol_data::FixedArray<#alloy_type, #size>),
                        format!("{sol_name}[{size}]").into(),
                    ),
                    None => Self::new(
                        parse_quote!(stylus_sdk::alloy_sol_types::sol_data::Array<#alloy_type>),
                        format!("{sol_name}[]").into(),
                    ),
                }
            }
            Type::Tuple(tup) => {
                if tup.types.is_empty() {
                    Self::new(parse!("()"), "()".into())
                } else if tup.types.len() == 1 {
                    Self::from(&tup.types[0])
                } else {
                    let type_info = tup.types.iter().map(Self::from);
                    let alloy_types = type_info.clone().map(|info| info.alloy_type);
                    let alloy_type = parse_quote! {
                        (#(#alloy_types,)*)
                    };
                    let sol_names = type_info
                        .map(|info| info.sol_name.to_string())
                        .collect::<Vec<_>>()
                        .join(",");
                    Self::new(alloy_type, format!("({sol_names})").into())
                }
            }
            Type::Custom(path) => {
                let sol_path = path.to_string().into();
                let path = syn::Path {
                    leading_colon: None,
                    segments: path.iter().cloned().map(syn::PathSegment::from).collect(),
                };
                let ty = syn::TypePath { qself: None, path }.into();
                Self::new(ty, sol_path)
            }
            _ => todo!("Solidity type {ty} is not yet implemented in sol_interface!"),
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::SolidityTypeInfo;

    macro_rules! sol_type_test {
        ($sol:ident, $alloy:ty) => {
            sol_type_test!($sol, stringify!($sol), $alloy);
        };
        ($name:ident, $sol:expr, $alloy:ty) => {
            sol_type_test!($name, $sol, @parse_quote!($alloy));
        };
        ($name:ident, $sol:expr, @$alloy:expr) => {
            paste::paste! {
                #[test]
                fn [<test_sol_ $name>]() {
                    let info = SolidityTypeInfo::from(&syn::parse_str($sol).unwrap());
                    assert_eq!(info.sol_name, $sol);
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
    sol_type_test!(
        array,
        "int256[]",
        stylus_sdk::alloy_sol_types::sol_data::Array<
            stylus_sdk::alloy_sol_types::sol_data::Int<256>,
        >
    );
    sol_type_test!(
        fixed_array,
        "int256[100]",
        stylus_sdk::alloy_sol_types::sol_data::FixedArray<
            stylus_sdk::alloy_sol_types::sol_data::Int<256>,
            100usize,
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
