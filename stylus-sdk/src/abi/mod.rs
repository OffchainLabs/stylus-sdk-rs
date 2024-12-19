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
use alloy_primitives::U256;
use core::borrow::BorrowMut;

use alloy_sol_types::{abi::TokenSeq, private::SolTypeValue, SolType};

use crate::{
    console,
    storage::{StorageType, TopLevelStorage},
    ArbResult,
};

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

    /// Receive function for this contract. Called when no calldata is provided.
    /// A receive function may not be defined, in which case this method will return None.
    /// Receive functions are always payable, take in no inputs, and return no outputs.
    /// If defined, they will always be called when a transaction does not send any
    /// calldata, regardless of the transaction having a value attached.
    fn receive(storage: &mut S) -> Option<()>;

    /// Called when no receive function is defined.
    /// If no #[fallback] function is defined in the contract, then any transactions that do not
    /// match a selector will revert.
    /// A fallback function may have two different implementations. It can be either declared
    /// without any input or output, or with bytes input calldata and bytes output. If a user
    /// defines a fallback function with no input or output, then this method will be called
    /// and the underlying user-defined function will simply be invoked with no input.
    /// A fallback function can be declared as payable. If not payable, then any transactions
    /// that trigger a fallback with value attached will revert.
    fn fallback(storage: &mut S, calldata: &[u8]) -> Option<ArbResult>;

    /// The router_entrypoint calls the constructor when the selector is CONSTRUCTOR_SELECTOR.
    /// The implementation should: decode the calldata and pass the parameters to the user-defined
    /// constructor; and call internal::constructor_guard to ensure it is only executed once.
    /// Since each constructor has its own set of parameters, this function won't call the
    /// constructors for inherited structs automatically. Instead, the user-defined function should
    /// call the base classes constructors.
    /// A constructor function can be declared as payable. If not payable, then any transactions
    /// that trigger the constructor with value attached will revert.
    fn constructor(storage: &mut S, calldata: &[u8]) -> Option<ArbResult>;
}

/// Entrypoint used when `#[entrypoint]` is used on a contract struct.
/// Solidity requires specific routing logic for situations in which no function selector
/// matches the input calldata in the form of two different functions named "receive" and "fallback".
/// The purity and type definitions are as follows:
///
/// - receive takes no input data, returns no data, and is always payable.
/// - fallback offers two possible implementations. It can be either declared without input or return
//    parameters, or with input bytes calldata and return bytes memory.
//
//  The fallback function MAY be payable. If not payable, then any transactions not matching any
//  other function which send value will revert.
//
//  The order of routing semantics for receive and fallback work as follows:
//
//  - If a receive function exists, it is always called whenever calldata is empty, even
//    if no value is received in the transaction. It is implicitly payable.
//  - Fallback is called when no other function matches a selector. If a receive function is not
//    defined, then calls with no input calldata will be routed to the fallback function.
pub fn router_entrypoint<R, S>(input: alloc::vec::Vec<u8>) -> ArbResult
where
    R: Router<S>,
    S: StorageType + TopLevelStorage + BorrowMut<R::Storage>,
{
    let mut storage = unsafe { S::new(U256::ZERO, 0) };

    if input.is_empty() {
        console!("no calldata provided");
        if R::receive(&mut storage).is_some() {
            return Ok(Vec::new());
        }
        // Try fallback function with no inputs if defined.
        if let Some(res) = R::fallback(&mut storage, &[]) {
            return res;
        }
        // Revert as no receive or fallback were defined.
        return Err(Vec::new());
    }

    if input.len() >= 4 {
        let selector = u32::from_be_bytes(TryInto::try_into(&input[..4]).unwrap());
        if selector == CONSTRUCTOR_SELECTOR {
            if let Some(res) = R::constructor(&mut storage, &input[4..]) {
                return res;
            }
        } else if let Some(res) = R::route(&mut storage, selector, &input[4..]) {
            return res;
        } else {
            console!("unknown method selector: {selector:08x}");
        }
    }

    // Try fallback function.
    if let Some(res) = R::fallback(&mut storage, &input) {
        return res;
    }

    Err(Vec::new())
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
    ($name:expr $(,)?) => {{
        const DIGEST: [u8; 32] = $crate::keccak_const::Keccak256::new()
            .update($name.as_bytes())
            .update(b"()")
            .finalize();
        $crate::abi::internal::digest_to_selector(DIGEST)
    }};

    ($name:expr, $first:ty $(, $ty:ty)* $(,)?) => {{
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

/// The function selector for Stylus constructors.
pub const CONSTRUCTOR_SELECTOR: u32 =
    u32::from_be_bytes(function_selector!(internal::CONSTRUCTOR_BASE_NAME));

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
