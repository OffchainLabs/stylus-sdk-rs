// Copyright 2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use super::{AbiType, ConstString};
use alloy_primitives::{Address, Signed, Uint};
use alloy_sol_types::sol_data::{self, IntBitCount, SupportedInt};

#[cfg(test)]
use alloy_primitives::FixedBytes;

/// Generates a test to ensure the two-way relationship between Rust Types and Sol Types is bijective.
macro_rules! test_type {
    ($name:tt, $as_arg:expr, $($ty:tt)*) => {
        #[cfg(test)]
        paste::paste! {
            #[allow(non_snake_case)]
            #[test]
            fn [<test_ $name>]() {
                assert_eq!(
                    <$($ty)* as AbiType>::ABI.as_str(),
                    <<$($ty)* as AbiType>::SolType as alloy_sol_types::SolType>::sol_type_name(),
                    "{}'s ABI didn't match its SolType sol_type_name",
                    stringify!($($ty)*),
                );
                assert_eq!(
                    <$($ty)* as AbiType>::EXPORT_ABI_ARG.as_str(),
                    $as_arg,
                );
            }
        }
    };
}

macro_rules! append {
    ($stem:expr, $leaf:expr) => {
        $stem.concat(ConstString::new($leaf))
    };
}

macro_rules! append_dec {
    ($stem:expr, $num:expr) => {
        ConstString::new($stem).concat(ConstString::from_decimal_number($num))
    };
}

test_type!(bytes, "bytes calldata", super::Bytes);

impl<const BITS: usize, const LIMBS: usize> AbiType for Uint<BITS, LIMBS>
where
    IntBitCount<BITS>: SupportedInt<Uint = Self>,
{
    type SolType = sol_data::Uint<BITS>;

    const ABI: ConstString = append_dec!("uint", BITS);
}

// test_type!(uint160, "uint160", Uint<160, 3>); TODO: audit alloy
test_type!(uint256, "uint256", Uint<256, 4>);

impl<const BITS: usize, const LIMBS: usize> AbiType for Signed<BITS, LIMBS>
where
    IntBitCount<BITS>: SupportedInt<Int = Self>,
{
    type SolType = sol_data::Int<BITS>;

    const ABI: ConstString = append_dec!("int", BITS);
}

// test_type!(int160, "int160", Signed<160, 3>); TODO: audit alloy
test_type!(int256, "int256", Signed<256, 4>);

macro_rules! impl_int {
    ($bits:literal, $as_arg:expr, $unsigned:ty, $signed:ty) => {
        impl AbiType for $unsigned
        where
            IntBitCount<$bits>: SupportedInt<Uint = Self>,
        {
            type SolType = sol_data::Uint<$bits>;

            const ABI: ConstString = append_dec!("uint", $bits);
        }

        impl AbiType for $signed
        where
            IntBitCount<$bits>: SupportedInt<Int = Self>,
        {
            type SolType = sol_data::Int<$bits>;

            const ABI: ConstString = append_dec!("int", $bits);
        }

        test_type!($unsigned, format!("u{}", $as_arg), $unsigned);
        test_type!($signed, $as_arg, $signed);
    };
}

impl_int!(8, "int8", u8, i8);
impl_int!(16, "int16", u16, i16);
impl_int!(32, "int32", u32, i32);
impl_int!(64, "int64", u64, i64);
impl_int!(128, "int128", u128, i128);

macro_rules! impl_alloy {
    ($rust_type:ty, $sol_type:ident $(<$generic:tt>)?, $signature:literal) => {
        impl AbiType for $rust_type {
            type SolType = sol_data::$sol_type $(<$generic>)*;

            const ABI: ConstString = ConstString::new($signature);
        }
        test_type!($signature, $signature, $rust_type);
    };
}

impl_alloy!(bool, Bool, "bool");
impl_alloy!(Address, Address, "address");

impl AbiType for String {
    type SolType = sol_data::String;

    const ABI: ConstString = ConstString::new("string");

    const EXPORT_ABI_ARG: ConstString = append!(Self::ABI, " calldata");

    const EXPORT_ABI_RET: ConstString = append!(Self::ABI, " memory");
}

impl<T: AbiType> AbiType for Vec<T> {
    type SolType = sol_data::Array<T::SolType>;

    const ABI: ConstString = append!(T::ABI, "[]");

    const EXPORT_ABI_ARG: ConstString = Self::EXPORT_ABI_RET; // vectors are never calldata

    const EXPORT_ABI_RET: ConstString = append!(T::ABI, "[] memory");

    const CAN_BE_CALLDATA: bool = false;
}

test_type!(vec_of_u8s, "uint8[] memory", Vec<u8>);
test_type!(
    vec_of_u256s,
    "uint256[] memory",
    Vec<alloy_primitives::U256>
);
test_type!(vec_of_bytes, "bytes[] memory", Vec<super::Bytes>);
test_type!(vec_of_fixed_bytes, "bytes18[] memory", Vec<FixedBytes<18>>);

impl<T: AbiType, const N: usize> AbiType for [T; N] {
    type SolType = sol_data::FixedArray<T::SolType, N>;

    const ABI: ConstString = T::ABI
        .concat(ConstString::new("["))
        .concat(ConstString::from_decimal_number(N))
        .concat(ConstString::new("]"));

    const EXPORT_ABI_ARG: ConstString = Self::ABI.concat(ConstString::select(
        T::CAN_BE_CALLDATA,
        " calldata",
        " memory",
    ));

    const EXPORT_ABI_RET: ConstString = append!(Self::ABI, " memory");

    const CAN_BE_CALLDATA: bool = T::CAN_BE_CALLDATA;
}

test_type!(array_of_bools, "bool[5] calldata", [bool; 5]);
test_type!(array_of_nested_u32s, "uint32[2][4] calldata", [[u32; 2]; 4]);
test_type!(
    array_of_fixed_bytes,
    "bytes32[] memory",
    Vec<FixedBytes<32>>
);

impl AbiType for () {
    type SolType = ();

    const ABI: ConstString = ConstString::new("()");
}

test_type!(empty_tuple, "()", ());

macro_rules! impl_tuple {
    () => {};
    ($first:ident $(, $rest:ident)*) => {
        impl<$first: AbiType $(, $rest: AbiType)*> AbiType for ( $first $(, $rest)* , ) {
            type SolType = ( $first::SolType $(, $rest::SolType)* , );

            const ABI: ConstString = ConstString::new("(")
                .concat($first::ABI)
                $(
                    .concat(ConstString::new(","))
                    .concat($rest::ABI)
                )*
                .concat(ConstString::new(")"));

            const EXPORT_ABI_ARG: ConstString = ConstString::new("(")
                .concat($first::EXPORT_ABI_ARG)
                $(
                    .concat(ConstString::new(", "))
                    .concat($rest::EXPORT_ABI_ARG)
                )*
                .concat(ConstString::new(")"));

            const EXPORT_ABI_RET: ConstString = ConstString::new("(")
                .concat($first::EXPORT_ABI_RET)
                $(
                    .concat(ConstString::new(", "))
                    .concat($rest::EXPORT_ABI_RET)
                )*
                .concat(ConstString::new(")"));

            const CAN_BE_CALLDATA: bool = false;
        }

        impl_tuple! { $($rest),* }
    };
}

impl_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);

test_type!(tuple_of_single_u8, "(uint8)", (u8,));
test_type!(tuple_of_single_u256, "(uint256)", (alloy_primitives::U256,));

test_type!(tuple_of_two_u8s, "(uint8, uint8)", (u8, u8));
test_type!(
    tuple_of_u8_and_u256,
    "(uint8, uint256)",
    (u8, alloy_primitives::U256)
);

test_type!(
    tuple_of_five_types,
    "(uint8, uint256[] memory, bytes calldata, bytes2, bool[][8] memory)",
    (
        u8,
        Vec<alloy_primitives::U256>,
        super::Bytes,
        FixedBytes<2>,
        [Vec<bool>; 8],
    )
);
