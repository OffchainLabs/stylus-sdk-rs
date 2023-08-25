// Copyright 2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use crate::{storage::TopLevelStorage, ArbResult};
use alloy_sol_types::SolType;
use core::borrow::BorrowMut;

pub use bytes::{Bytes, BytesSolType};
pub use const_string::ConstString;
pub use fixed_bytes::{FixedBytes, FixedBytesSolType};

#[cfg(feature = "export-abi")]
pub use export::GenerateAbi;

#[cfg(feature = "export-abi")]
pub mod export;

mod bytes;
mod const_string;
mod fixed_bytes;
mod impls;

#[doc(hidden)]
pub mod internal;

/// Executes a method given a selector and calldata.
/// This trait can be automatically implemented via `#[external]`.
/// Composition with other routers is possible via `#[inherit]`.
pub trait Router<S>
where
    S: TopLevelStorage + BorrowMut<Self::Storage>,
{
    type Storage;

    /// Tries to find and execute a method for the given selector, returning `None` if none is found.
    /// Routes add via `#[inherit]` will only execute if no match is found among `Self`.
    /// This means that it is possible to override a method by redefining it in `Self`.
    fn route(storage: &mut S, selector: u32, input: &[u8]) -> Option<ArbResult>;
}

/// Provides a mapping of Rust to Solidity types.
/// When combined with alloy, which provides the reverse direction, a two-way relationship is formed.
///
/// Additionally, `AbiType` provides a const equivalent to alloy's [`SolType::sol_type_name`].
pub trait AbiType {
    type SolType: SolType<RustType = Self>;

    /// Equivalent to [`SolType::sol_type_name`], but `const`.
    const ABI: ConstString;
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

#[test]
fn test_function_selector() {
    use alloy_primitives::{Address, U256};
    assert_eq!(u32::from_be_bytes(function_selector!("foo")), 0xc2985578);
    assert_eq!(function_selector!("foo", Address), [0xfd, 0xf8, 0x0b, 0xda]);

    const TEST_SELECTOR: [u8; 4] = function_selector!("foo", Address, U256);
    assert_eq!(TEST_SELECTOR, 0xbd0d639f_u32.to_be_bytes());
}
