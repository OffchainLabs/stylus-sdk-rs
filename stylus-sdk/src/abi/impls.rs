// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use super::{AbiType, ConstString};
use alloc::{string::String, vec::Vec};
use alloy_primitives::{Address, Bytes, FixedBytes};
use alloy_sol_types::{
    private::SolTypeValue,
    sol_data::{self, ByteCount, IntBitCount, SupportedFixedBytes, SupportedInt},
    SolType,
};

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
                    <<$($ty)* as AbiType>::SolType as alloy_sol_types::SolType>::SOL_NAME,
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

impl<const N: usize> AbiType for FixedBytes<N>
where
    ByteCount<N>: SupportedFixedBytes,
{
    type SolType = sol_data::FixedBytes<N>;

    const ABI: ConstString = append_dec!("bytes", N);
}

test_type!(uint160, "uint160", alloy_primitives::Uint<160, 3>);
test_type!(uint256, "uint256", alloy_primitives::Uint<256, 4>);

impl AbiType for Bytes {
    type SolType = sol_data::Bytes;

    const ABI: ConstString = ConstString::new("bytes");

    const EXPORT_ABI_ARG: ConstString = Self::ABI.concat(ConstString::new(" calldata"));

    const EXPORT_ABI_RET: ConstString = Self::ABI.concat(ConstString::new(" memory"));
}

test_type!(bytes, "bytes calldata", Bytes);
test_type!(sdk_bytes, "bytes calldata", super::Bytes);

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

    const SELECTOR_ABI: ConstString = append!(T::SELECTOR_ABI, "[]");

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

    const SELECTOR_ABI: ConstString = T::SELECTOR_ABI
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

            const SELECTOR_ABI: ConstString = ConstString::new("(")
                .concat($first::SELECTOR_ABI)
                $(
                    .concat(ConstString::new(","))
                    .concat($rest::SELECTOR_ABI)
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

            fn abi_encode_return<RustTy: ?Sized + SolTypeValue<Self::SolType>>(rust: &RustTy) -> Vec<u8> {
                Self::SolType::abi_encode_params(rust)
            }
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

#[cfg(test)]
mod tests {
    use alloy_primitives::{hex, FixedBytes, U256};

    use crate::abi::{test_encode_decode_params, Bytes};

    #[test]
    fn encode_tuple_of_single_u8() {
        let value = (100u8,);
        let encoded = hex!("0000000000000000000000000000000000000000000000000000000000000064");
        test_encode_decode_params(value, encoded)
    }

    #[test]
    fn encode_tuple_of_single_u256() {
        let value = (U256::from(100),);
        let encoded = hex!("0000000000000000000000000000000000000000000000000000000000000064");
        test_encode_decode_params(value, encoded)
    }

    #[test]
    fn encode_tuple_of_two_u8s() {
        let value = (100u8, 200u8);
        let encoded = hex!(
            "0000000000000000000000000000000000000000000000000000000000000064"
            "00000000000000000000000000000000000000000000000000000000000000C8"
        );
        test_encode_decode_params(value, encoded)
    }

    #[test]
    fn encode_tuple_of_u8_and_u256() {
        let value = (100u8, U256::from(200));
        let encoded = hex!(
            "0000000000000000000000000000000000000000000000000000000000000064"
            "00000000000000000000000000000000000000000000000000000000000000C8"
        );
        test_encode_decode_params(value, encoded)
    }

    #[test]
    fn encode_tuple_of_five_types() {
        let value = (
            100u8,
            vec![U256::from(1), U256::from(2)],
            Bytes::from(vec![1, 2, 3, 4]),
            FixedBytes::new([5, 6]),
            [vec![true, false, true], vec![false, true, false]],
        );
        let encoded = hex!(
            "0000000000000000000000000000000000000000000000000000000000000064"
            "00000000000000000000000000000000000000000000000000000000000000A0"
            "0000000000000000000000000000000000000000000000000000000000000100"
            "0506000000000000000000000000000000000000000000000000000000000000"
            "0000000000000000000000000000000000000000000000000000000000000140"
            "0000000000000000000000000000000000000000000000000000000000000002"
            "0000000000000000000000000000000000000000000000000000000000000001"
            "0000000000000000000000000000000000000000000000000000000000000002"
            "0000000000000000000000000000000000000000000000000000000000000004"
            "0102030400000000000000000000000000000000000000000000000000000000"
            "0000000000000000000000000000000000000000000000000000000000000040"
            "00000000000000000000000000000000000000000000000000000000000000C0"
            "0000000000000000000000000000000000000000000000000000000000000003"
            "0000000000000000000000000000000000000000000000000000000000000001"
            "0000000000000000000000000000000000000000000000000000000000000000"
            "0000000000000000000000000000000000000000000000000000000000000001"
            "0000000000000000000000000000000000000000000000000000000000000003"
            "0000000000000000000000000000000000000000000000000000000000000000"
            "0000000000000000000000000000000000000000000000000000000000000001"
            "0000000000000000000000000000000000000000000000000000000000000000"
        );
        test_encode_decode_params(value, encoded)
    }

    #[test]
    fn encode_decode_empty_bytes() {
        test_encode_decode_params(
            (Bytes::new(),),
            hex!(
                "0000000000000000000000000000000000000000000000000000000000000020"
                "0000000000000000000000000000000000000000000000000000000000000000"
            ),
        );
    }

    #[test]
    fn encode_decode_one_byte() {
        test_encode_decode_params(
            (Bytes::from(vec![100]),),
            hex!(
                "0000000000000000000000000000000000000000000000000000000000000020"
                "0000000000000000000000000000000000000000000000000000000000000001"
                "6400000000000000000000000000000000000000000000000000000000000000"
            ),
        );
    }

    #[test]
    fn encode_decode_several_bytes() {
        let mut input = Vec::with_capacity(40);
        input.extend([1, 2, 3, 4]);
        input.extend([0u8; 32]);
        input.extend([5, 6, 7, 8]);
        let value = (Bytes::from(input),);
        let encoded = hex!(
            "0000000000000000000000000000000000000000000000000000000000000020"
            "0000000000000000000000000000000000000000000000000000000000000028"
            "0102030400000000000000000000000000000000000000000000000000000000"
            "0000000005060708000000000000000000000000000000000000000000000000"
        );
        test_encode_decode_params(value, encoded);
    }

    #[test]
    fn encode_decode_bytes_tuple() {
        let mut input = Vec::with_capacity(40);
        input.extend([1, 2, 3, 4]);
        input.extend([0u8; 32]);
        input.extend([5, 6, 7, 8]);
        let value = (
            Bytes::from(input),
            Bytes::new(),
            Bytes::from(vec![1, 2, 3, 4]),
        );

        let encoded = hex!(
            "0000000000000000000000000000000000000000000000000000000000000060"
            "00000000000000000000000000000000000000000000000000000000000000C0"
            "00000000000000000000000000000000000000000000000000000000000000E0"
            "0000000000000000000000000000000000000000000000000000000000000028"
            "0102030400000000000000000000000000000000000000000000000000000000"
            "0000000005060708000000000000000000000000000000000000000000000000"
            "0000000000000000000000000000000000000000000000000000000000000000"
            "0000000000000000000000000000000000000000000000000000000000000004"
            "0102030400000000000000000000000000000000000000000000000000000000"
        );

        test_encode_decode_params(value, encoded)
    }
}
