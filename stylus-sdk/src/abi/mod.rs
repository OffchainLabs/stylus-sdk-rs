// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Solidity ABIs for Rust types.
//!
//! Alloy provides a 1-way mapping of Solidity types to Rust ones via [`SolType`].
//! This module provides the inverse mapping, forming a bijective, 2-way relationship between Rust and Solidity.
//!
//! This allows the [`prelude`][prelude] macros to generate method selectors, export
//! Solidity interfaces, and otherwise facilitate inter-op between Rust and Solidity contracts.
//!
//! Notably, the SDK treats `Vec<u8>` as a Solidity `uint8[]`.
//! For a Solidity `bytes`, see [`Bytes`].
//!
//! [prelude]: crate::prelude
//!

use alloc::vec::Vec;
use core::borrow::BorrowMut;

use alloy_sol_types::{abi::TokenSeq, private::SolTypeValue, SolType};

use crate::{storage::TopLevelStorage, ArbResult};

pub use bytes::{Bytes, BytesSolType};
pub use const_string::ConstString;

#[cfg(feature = "export-abi")]
pub use export::GenerateAbi;

#[cfg(feature = "export-abi")]
pub mod export;

mod bytes;
mod const_string;
mod impls;
mod ints;

#[doc(hidden)]
pub mod internal;

/// Executes a method given a selector and calldata.
/// This trait can be automatically implemented via `#[public]`.
/// Composition with other routers is possible via `#[inherit]`.
pub trait Router<S>
where
    S: TopLevelStorage + BorrowMut<Self::Storage>,
{
    /// The type the [`TopLevelStorage`] borrows into. Usually just `Self`.
    type Storage;

    /// Tries to find and execute a method for the given selector, returning `None` if none is found.
    /// Routes add via `#[inherit]` will only execute if no match is found among `Self`.
    /// This means that it is possible to override a method by redefining it in `Self`.
    fn route(storage: &mut S, selector: u32, input: &[u8]) -> Option<ArbResult>;
}

/// Provides a mapping of Rust to Solidity types.
/// When combined with alloy, which provides the reverse direction, a two-way relationship is formed.
///
/// Additionally, `AbiType` provides a `const` equivalent to alloy's [`SolType::sol_type_name`].
pub trait AbiType {
    /// The associated Solidity type.
    type SolType: SolType<RustType = Self>;

    /// Equivalent to [`SolType::sol_type_name`], but `const`.
    const ABI: ConstString;

    /// String to use when the type is an interface method argument.
    const EXPORT_ABI_ARG: ConstString = Self::ABI;

    /// String to use when the type is an interface method return value.
    const EXPORT_ABI_RET: ConstString = Self::ABI;

    /// Whether the type is allowed in calldata
    const CAN_BE_CALLDATA: bool = true;
}

/// Generates a function selector for the given method and its args.
#[macro_export]
macro_rules! function_selector {
    ($name:literal $(,)?) => {{
        const DIGEST: [u8; 32] = $crate::keccak_const::Keccak256::new()
            .update($name.as_bytes())
            .update(b"()")
            .finalize();
        $crate::abi::internal::digest_to_selector(DIGEST)
    }};

    ($name:literal, $first:ty $(, $ty:ty)* $(,)?) => {{
        const DIGEST: [u8; 32] = $crate::keccak_const::Keccak256::new()
            .update($name.as_bytes())
            .update(b"(")
            .update(<$first as $crate::abi::AbiType>::ABI.as_bytes())
            $(
                .update(b",")
                .update(<$ty as $crate::abi::AbiType>::ABI.as_bytes())
            )*
            .update(b")")
            .finalize();
        $crate::abi::internal::digest_to_selector(DIGEST)
    }};
}

/// ABI decode a tuple of parameters
pub fn decode_params<T>(data: &[u8]) -> alloy_sol_types::Result<T>
where
    T: AbiType + SolTypeValue<<T as AbiType>::SolType>,
    for<'a> <<T as AbiType>::SolType as SolType>::Token<'a>: TokenSeq<'a>,
{
    T::SolType::abi_decode_params(data, true)
}

/// ABI encode a value
pub fn encode<T>(value: &T) -> Vec<u8>
where
    T: AbiType + SolTypeValue<<T as AbiType>::SolType>,
{
    T::SolType::abi_encode(value)
}

/// ABI encode a tuple of parameters
pub fn encode_params<T>(value: &T) -> Vec<u8>
where
    T: AbiType + SolTypeValue<<T as AbiType>::SolType>,
    for<'a> <<T as AbiType>::SolType as SolType>::Token<'a>: TokenSeq<'a>,
{
    T::SolType::abi_encode_params(value)
}

/// Encoded size of some sol type
pub fn encoded_size<T>(value: &T) -> usize
where
    T: AbiType + SolTypeValue<<T as AbiType>::SolType>,
{
    T::SolType::abi_encoded_size(value)
}

/// Parform a test of both the encode and decode functions for a given type
///
/// This is intended for use within unit tests.
#[cfg(test)]
fn test_encode_decode_params<T, B>(value: T, buffer: B)
where
    T: core::fmt::Debug + PartialEq + AbiType + SolTypeValue<<T as AbiType>::SolType>,
    for<'a> <<T as AbiType>::SolType as SolType>::Token<'a>: TokenSeq<'a>,
    B: core::fmt::Debug + AsRef<[u8]>,
{
    let encoded = encode_params(&value);
    assert_eq!(encoded, buffer.as_ref());

    let decoded = decode_params::<T>(buffer.as_ref()).unwrap();
    assert_eq!(decoded, value);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_function_selector() {
        use alloy_primitives::{Address, U256};
        assert_eq!(u32::from_be_bytes(function_selector!("foo")), 0xc2985578);
        assert_eq!(function_selector!("foo", Address), [0xfd, 0xf8, 0x0b, 0xda]);

        const TEST_SELECTOR: [u8; 4] = function_selector!("foo", Address, U256);
        assert_eq!(TEST_SELECTOR, 0xbd0d639f_u32.to_be_bytes());
    }
}
