// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

//! Support for generic integer types found in [alloy_primitives].

use alloy_primitives::{Signed, Uint};
use alloy_sol_types::{
    private::SolTypeValue,
    sol_data::{self, IntBitCount, SupportedInt},
    SolType,
};

use super::{AbiType, ConstString};

macro_rules! impl_alloy_int {
    ($BITS:expr, $LIMBS:expr) => {
        impl_alloy_int!($BITS, $LIMBS, sol_data::Uint<$BITS>, sol_data::Int<$BITS>);
    };
    (@$BITS:expr, $LIMBS:expr) => {
        impl_alloy_int!($BITS, $LIMBS, OverloadUint<$BITS, $LIMBS>, OverloadInt<$BITS, $LIMBS>);
    };
    ($BITS:expr, $LIMBS:expr, $uint_ty:ty, $int_ty:ty) => {
        impl AbiType for Uint<$BITS, $LIMBS> {
            type SolType = $uint_ty;

            const ABI: ConstString = ConstString::new(Self::SolType::SOL_NAME);
        }

        impl AbiType for Signed<$BITS, $LIMBS> {
            type SolType = $int_ty;

            const ABI: ConstString = ConstString::new(Self::SolType::SOL_NAME);
        }
    };
}

impl_alloy_int!(@8, 1);
impl_alloy_int!(@16, 1);
impl_alloy_int!(24, 1);
impl_alloy_int!(@32, 1);
impl_alloy_int!(40, 1);
impl_alloy_int!(48, 1);
impl_alloy_int!(56, 1);
impl_alloy_int!(@64, 1);
impl_alloy_int!(72, 2);
impl_alloy_int!(80, 2);
impl_alloy_int!(88, 2);
impl_alloy_int!(96, 2);
impl_alloy_int!(104, 2);
impl_alloy_int!(112, 2);
impl_alloy_int!(120, 2);
impl_alloy_int!(@128, 2);
impl_alloy_int!(136, 3);
impl_alloy_int!(144, 3);
impl_alloy_int!(152, 3);
impl_alloy_int!(160, 3);
impl_alloy_int!(168, 3);
impl_alloy_int!(176, 3);
impl_alloy_int!(184, 3);
impl_alloy_int!(192, 3);
impl_alloy_int!(200, 4);
impl_alloy_int!(208, 4);
impl_alloy_int!(216, 4);
impl_alloy_int!(224, 4);
impl_alloy_int!(232, 4);
impl_alloy_int!(240, 4);
impl_alloy_int!(248, 4);
impl_alloy_int!(256, 4);

pub struct OverloadInt<const BITS: usize, const LIMBS: usize>;

impl<const BITS: usize, const LIMBS: usize> SolType for OverloadInt<BITS, LIMBS>
where
    IntBitCount<BITS>: SupportedInt,
    Self: ConvertInt<
        Int = <sol_data::Int<BITS> as SolType>::RustType,
        AlloyInt = Signed<BITS, LIMBS>,
    >,
{
    type RustType = Signed<BITS, LIMBS>;
    type Token<'a> = <sol_data::Int<BITS> as SolType>::Token<'a>;

    const SOL_NAME: &'static str = <sol_data::Int<BITS> as SolType>::SOL_NAME;
    const ENCODED_SIZE: Option<usize> = <sol_data::Int<BITS> as SolType>::ENCODED_SIZE;
    const PACKED_ENCODED_SIZE: Option<usize> =
        <sol_data::Int<BITS> as SolType>::PACKED_ENCODED_SIZE;
    const DYNAMIC: bool = <sol_data::Int<BITS> as SolType>::DYNAMIC;

    #[inline]
    fn valid_token(token: &Self::Token<'_>) -> bool {
        <sol_data::Int<BITS> as SolType>::valid_token(token)
    }

    #[inline]
    fn detokenize(token: Self::Token<'_>) -> Self::RustType {
        <Self as ConvertInt>::to_alloy(<sol_data::Int<BITS> as SolType>::detokenize(token))
    }
}

impl<const BITS: usize, const LIMBS: usize> SolTypeValue<OverloadInt<BITS, LIMBS>>
    for Signed<BITS, LIMBS>
where
    IntBitCount<BITS>: SupportedInt,
    OverloadInt<BITS, LIMBS>: ConvertInt<
        Int = <sol_data::Int<BITS> as SolType>::RustType,
        AlloyInt = Signed<BITS, LIMBS>,
    >,
{
    #[inline]
    fn stv_abi_encoded_size(&self) -> usize {
        <OverloadInt<BITS, LIMBS> as ConvertInt>::to_int(*self).stv_abi_encoded_size()
    }

    #[inline]
    fn stv_to_tokens(&self) -> <OverloadInt<BITS, LIMBS> as SolType>::Token<'_> {
        <OverloadInt<BITS, LIMBS> as ConvertInt>::to_int(*self).stv_to_tokens()
    }

    #[inline]
    fn stv_abi_packed_encoded_size(&self) -> usize {
        <OverloadInt<BITS, LIMBS> as ConvertInt>::to_int(*self).stv_abi_packed_encoded_size()
    }

    #[inline]
    fn stv_abi_encode_packed_to(&self, out: &mut alloc::vec::Vec<u8>) {
        <OverloadInt<BITS, LIMBS> as ConvertInt>::to_int(*self).stv_abi_encode_packed_to(out)
    }

    #[inline]
    fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
        <OverloadInt<BITS, LIMBS> as ConvertInt>::to_int(*self).stv_eip712_data_word()
    }
}

pub struct OverloadUint<const BITS: usize, const LIMBS: usize>;

impl<const BITS: usize, const LIMBS: usize> SolType for OverloadUint<BITS, LIMBS>
where
    IntBitCount<BITS>: SupportedInt,
    Self: ConvertInt<
        Int = <sol_data::Uint<BITS> as SolType>::RustType,
        AlloyInt = Uint<BITS, LIMBS>,
    >,
{
    type RustType = Uint<BITS, LIMBS>;
    type Token<'a> = <sol_data::Uint<BITS> as SolType>::Token<'a>;

    const SOL_NAME: &'static str = <sol_data::Uint<BITS> as SolType>::SOL_NAME;
    const ENCODED_SIZE: Option<usize> = <sol_data::Uint<BITS> as SolType>::ENCODED_SIZE;
    const PACKED_ENCODED_SIZE: Option<usize> =
        <sol_data::Uint<BITS> as SolType>::PACKED_ENCODED_SIZE;
    const DYNAMIC: bool = <sol_data::Uint<BITS> as SolType>::DYNAMIC;

    #[inline]
    fn valid_token(token: &Self::Token<'_>) -> bool {
        <sol_data::Int<BITS> as SolType>::valid_token(token)
    }

    #[inline]
    fn detokenize(token: Self::Token<'_>) -> Self::RustType {
        <Self as ConvertInt>::to_alloy(<sol_data::Uint<BITS> as SolType>::detokenize(token))
    }
}

impl<const BITS: usize, const LIMBS: usize> SolTypeValue<OverloadUint<BITS, LIMBS>>
    for Uint<BITS, LIMBS>
where
    IntBitCount<BITS>: SupportedInt,
    OverloadUint<BITS, LIMBS>: ConvertInt<
        Int = <sol_data::Uint<BITS> as SolType>::RustType,
        AlloyInt = Uint<BITS, LIMBS>,
    >,
{
    #[inline]
    fn stv_abi_encoded_size(&self) -> usize {
        <OverloadUint<BITS, LIMBS> as ConvertInt>::to_int(*self).stv_abi_encoded_size()
    }

    #[inline]
    fn stv_to_tokens(&self) -> <OverloadUint<BITS, LIMBS> as SolType>::Token<'_> {
        <OverloadUint<BITS, LIMBS> as ConvertInt>::to_int(*self).stv_to_tokens()
    }

    #[inline]
    fn stv_abi_packed_encoded_size(&self) -> usize {
        <OverloadUint<BITS, LIMBS> as ConvertInt>::to_int(*self).stv_abi_packed_encoded_size()
    }

    #[inline]
    fn stv_abi_encode_packed_to(&self, out: &mut alloc::vec::Vec<u8>) {
        <OverloadUint<BITS, LIMBS> as ConvertInt>::to_int(*self).stv_abi_encode_packed_to(out)
    }

    #[inline]
    fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
        <OverloadUint<BITS, LIMBS> as ConvertInt>::to_int(*self).stv_eip712_data_word()
    }
}

trait ConvertInt {
    type Int: TryInto<Self::AlloyInt>;
    type AlloyInt: TryInto<Self::Int>;
    fn to_int(value: Self::AlloyInt) -> Self::Int {
        value.try_into().map_err(|_| "int conversion error").unwrap()
    }
    fn to_alloy(value: Self::Int) -> Self::AlloyInt {
        value.try_into().map_err(|_| "int conversion error").unwrap()
    }
}

impl ConvertInt for OverloadUint<8, 1> {
    type Int = u8;
    type AlloyInt = Uint<8, 1>;
}

impl ConvertInt for OverloadUint<16, 1> {
    type Int = u16;
    type AlloyInt = Uint<16, 1>;
}

impl ConvertInt for OverloadUint<32, 1> {
    type Int = u32;
    type AlloyInt = Uint<32, 1>;
}

impl ConvertInt for OverloadUint<64, 1> {
    type Int = u64;
    type AlloyInt = Uint<64, 1>;
}

impl ConvertInt for OverloadUint<128, 2> {
    type Int = u128;
    type AlloyInt = Uint<128, 2>;
}

impl ConvertInt for OverloadInt<8, 1> {
    type Int = i8;
    type AlloyInt = Signed<8, 1>;
}

impl ConvertInt for OverloadInt<16, 1> {
    type Int = i16;
    type AlloyInt = Signed<16, 1>;
}

impl ConvertInt for OverloadInt<32, 1> {
    type Int = i32;
    type AlloyInt = Signed<32, 1>;
}

impl ConvertInt for OverloadInt<64, 1> {
    type Int = i64;
    type AlloyInt = Signed<64, 1>;
}

impl ConvertInt for OverloadInt<128, 2> {
    type Int = i128;
    type AlloyInt = Signed<128, 2>;
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{hex, Uint};

    use crate::abi::test_encode_decode_params;

    #[test]
    fn encode_decode_u24() {
        let value = (Uint::<24, 1>::from(10),);
        let encoded = hex!("000000000000000000000000000000000000000000000000000000000000000A");
        test_encode_decode_params(value, encoded);
    }

    #[test]
    fn encode_decode_u160() {
        let value = (Uint::<160, 3>::from(999),);
        let encoded = hex!("00000000000000000000000000000000000000000000000000000000000003E7");
        test_encode_decode_params(value, encoded);
    }
}
