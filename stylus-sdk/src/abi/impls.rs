// Copyright 2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use super::{AbiType, ConstString};
use alloy_primitives::{Address, Signed, Uint};
use alloy_sol_types::sol_data::{self, IntBitCount, SupportedInt};

/// Generates a test to ensure the two-way relationship between Rust Types and Sol Types is bijective.
macro_rules! test_type {
    ($name:tt, $($ty:tt)*) => {
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
            }
        }
    };
}

macro_rules! concat {
    ($stem:expr, $num:expr) => {
        ConstString::new($stem).concat(ConstString::from_decimal_number($num))
    };
}

test_type!(bytes, super::Bytes);

impl<const BITS: usize, const LIMBS: usize> AbiType for Uint<BITS, LIMBS>
where
    IntBitCount<BITS>: SupportedInt<Uint = Self>,
{
    type SolType = sol_data::Uint<BITS>;

    const ABI: ConstString = concat!("uint", BITS);
}

test_type!(uint160, Uint<256, 4>);
test_type!(uint256, Uint<256, 4>);

impl<const BITS: usize, const LIMBS: usize> AbiType for Signed<BITS, LIMBS>
where
    IntBitCount<BITS>: SupportedInt<Int = Self>,
{
    type SolType = sol_data::Int<BITS>;

    const ABI: ConstString = concat!("int", BITS);
}

test_type!(int160, Signed<256, 4>);
test_type!(int256, Signed<256, 4>);

macro_rules! impl_int {
    ($bits:literal, $unsigned:ty, $signed:ty) => {
        impl AbiType for $unsigned
        where
            IntBitCount<$bits>: SupportedInt<Uint = Self>,
        {
            type SolType = sol_data::Uint<$bits>;

            const ABI: ConstString = concat!("uint", $bits);
        }

        impl AbiType for $signed
        where
            IntBitCount<$bits>: SupportedInt<Int = Self>,
        {
            type SolType = sol_data::Int<$bits>;

            const ABI: ConstString = concat!("int", $bits);
        }

        test_type!($unsigned, $unsigned);
        test_type!($signed, $signed);
    };
}

impl_int!(8, u8, i8);
impl_int!(16, u16, i16);
impl_int!(32, u32, i32);
impl_int!(64, u64, i64);
impl_int!(128, u128, i128);

macro_rules! impl_alloy {
    ($rust_type:ty, $sol_type:ident $(<$generic:tt>)?, $signature:literal) => {
        impl AbiType for $rust_type {
            type SolType = sol_data::$sol_type $(<$generic>)*;

            const ABI: ConstString = ConstString::new($signature);
        }
        test_type!($signature, $rust_type);
    };
}

impl_alloy!(bool, Bool, "bool");
impl_alloy!(Address, Address, "address");
impl_alloy!(String, String, "string");

impl<T: AbiType> AbiType for Vec<T> {
    type SolType = sol_data::Array<T::SolType>;

    const ABI: ConstString = T::ABI.concat(ConstString::new("[]"));
}

test_type!(vec_of_u8s, Vec<u8>);
test_type!(vec_of_u256s, Vec<alloy_primitives::U256>);
test_type!(vec_of_bytes, Vec<super::Bytes>);
test_type!(vec_of_fixed_bytes, Vec<super::FixedBytes<18>>);

impl<T: AbiType, const N: usize> AbiType for [T; N] {
    type SolType = sol_data::FixedArray<T::SolType, N>;

    const ABI: ConstString = T::ABI
        .concat(ConstString::new("["))
        .concat(ConstString::from_decimal_number(N))
        .concat(ConstString::new("]"));
}

test_type!(array_of_bools, [bool; 5]);
test_type!(array_of_nested_u32s, [[u32; 2]; 4]);
test_type!(array_of_fixed_bytes, Vec<super::FixedBytes<32>>);

impl AbiType for () {
    type SolType = ();

    const ABI: ConstString = ConstString::new("()");
}

test_type!(empty_tuple, ());

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
        }

        impl_tuple! { $($rest),* }
    };
}

impl_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);

test_type!(tuple_of_single_u8, (u8,));
test_type!(tuple_of_single_u256, (alloy_primitives::U256,));

test_type!(tuple_of_two_u8s, (u8, u8));
test_type!(tuple_of_u8_and_u256, (u8, alloy_primitives::U256));

test_type!(
    tuple_of_four_types,
    (
        u8,
        Vec<alloy_primitives::U256>,
        super::Bytes,
        super::FixedBytes<2>
    )
);
