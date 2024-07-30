// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

//! Support for generic integer types, found in [alloy_primitives].

use alloy_primitives::{ruint::UintTryFrom, Signed, Uint};
use alloy_sol_types::{
    abi::token::WordToken,
    private::SolTypeValue,
    sol_data::{self, IntBitCount, SupportedInt},
    SolType,
};

use super::{AbiType, ConstString};

/// Represents [`intX`] in Solidity
///
/// A custom type is used here in lieu of [alloy_sol_types::sol_data::Int] in order to keep our
/// bijectivity constraint.
///
/// [`intX`]: https://docs.soliditylang.org/en/latest/types.html#integers
pub struct SolInt<const BITS: usize, const LIMBS: usize>;

impl<const BITS: usize, const LIMBS: usize> AbiType for Signed<BITS, LIMBS>
where
    for<'a> sol_data::Int<BITS>: SolType<Token<'a> = WordToken>,
    Converted<Signed<BITS, LIMBS>>: From<<sol_data::Int<BITS> as SolType>::RustType>,
    IntBitCount<BITS>: SupportedInt,
    Converted<<IntBitCount<BITS> as SupportedInt>::Int>: From<Signed<BITS, LIMBS>>,
{
    type SolType = SolInt<BITS, LIMBS>;

    const ABI: ConstString = ConstString::new(Self::SolType::SOL_NAME);
}

impl<const BITS: usize, const LIMBS: usize> SolType for SolInt<BITS, LIMBS>
where
    for<'a> sol_data::Int<BITS>: SolType<Token<'a> = WordToken>,
    Converted<Signed<BITS, LIMBS>>: From<<sol_data::Int<BITS> as SolType>::RustType>,
    IntBitCount<BITS>: SupportedInt,
    Converted<<IntBitCount<BITS> as SupportedInt>::Int>: From<Signed<BITS, LIMBS>>,
{
    type RustType = Signed<BITS, LIMBS>;
    type Token<'a> = WordToken;

    const SOL_NAME: &'static str = ConstString::new("int")
        .concat(ConstString::from_decimal_number(BITS))
        .as_str();
    const ENCODED_SIZE: Option<usize> = Some(32);

    #[inline]
    fn valid_token(token: &Self::Token<'_>) -> bool {
        sol_data::Int::<BITS>::valid_token(token)
    }

    #[inline]
    fn detokenize(token: Self::Token<'_>) -> Self::RustType {
        let converted: Converted<_> = sol_data::Int::<BITS>::detokenize(token).into();
        converted.0
    }
}

impl<const BITS: usize, const LIMBS: usize> SolTypeValue<SolInt<BITS, LIMBS>>
    for Signed<BITS, LIMBS>
where
    for<'a> sol_data::Int<BITS>: SolType<Token<'a> = WordToken>,
    Converted<Signed<BITS, LIMBS>>: From<<sol_data::Int<BITS> as SolType>::RustType>,
    IntBitCount<BITS>: SupportedInt,
    Converted<<IntBitCount<BITS> as SupportedInt>::Int>: From<Signed<BITS, LIMBS>>,
{
    #[inline]
    fn stv_to_tokens(&self) -> WordToken {
        let stv: Converted<<IntBitCount<BITS> as SupportedInt>::Int> = (*self).into();
        SolTypeValue::<sol_data::Int<BITS>>::stv_to_tokens(&stv.0)
    }

    #[inline]
    fn stv_abi_encode_packed_to(&self, out: &mut alloy_sol_types::private::Vec<u8>) {
        let stv: Converted<<IntBitCount<BITS> as SupportedInt>::Int> = (*self).into();
        SolTypeValue::<sol_data::Int<BITS>>::stv_abi_encode_packed_to(&stv.0, out)
    }

    #[inline]
    fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
        self.stv_to_tokens().0
    }
}

/// Represents [`uintX`] in Solidity
///
/// A custom type is used here in lieu of [alloy_sol_types::sol_data::Uint] in order to keep our
/// bijectivity constraint.
///
/// [`uintX`]: https://docs.soliditylang.org/en/latest/types.html#integers
pub struct SolUint<const BITS: usize, const LIMBS: usize>;

impl<const BITS: usize, const LIMBS: usize> AbiType for Uint<BITS, LIMBS>
where
    for<'a> sol_data::Uint<BITS>: SolType<Token<'a> = WordToken>,
    Uint<BITS, LIMBS>: UintTryFrom<<sol_data::Uint<BITS> as SolType>::RustType>,
    IntBitCount<BITS>: SupportedInt,
    Converted<<IntBitCount<BITS> as SupportedInt>::Uint>: From<Uint<BITS, LIMBS>>,
{
    type SolType = SolUint<BITS, LIMBS>;

    const ABI: ConstString = ConstString::new(Self::SolType::SOL_NAME);
}

impl<const BITS: usize, const LIMBS: usize> SolType for SolUint<BITS, LIMBS>
where
    for<'a> sol_data::Uint<BITS>: SolType<Token<'a> = WordToken>,
    Uint<BITS, LIMBS>: UintTryFrom<<sol_data::Uint<BITS> as SolType>::RustType>,
    IntBitCount<BITS>: SupportedInt,
    Converted<<IntBitCount<BITS> as SupportedInt>::Uint>: From<Uint<BITS, LIMBS>>,
{
    type RustType = Uint<BITS, LIMBS>;
    type Token<'a> = WordToken;

    const SOL_NAME: &'static str = ConstString::new("uint")
        .concat(ConstString::from_decimal_number(BITS))
        .as_str();
    const ENCODED_SIZE: Option<usize> = Some(32);

    #[inline]
    fn valid_token(token: &Self::Token<'_>) -> bool {
        sol_data::Uint::<BITS>::valid_token(token)
    }

    #[inline]
    fn detokenize(token: Self::Token<'_>) -> Self::RustType {
        Uint::from(sol_data::Uint::<BITS>::detokenize(token))
    }
}

impl<const BITS: usize, const LIMBS: usize> SolTypeValue<SolUint<BITS, LIMBS>> for Uint<BITS, LIMBS>
where
    for<'a> sol_data::Uint<BITS>: SolType<Token<'a> = WordToken>,
    Uint<BITS, LIMBS>: UintTryFrom<<sol_data::Uint<BITS> as SolType>::RustType>,
    IntBitCount<BITS>: SupportedInt,
    Converted<<IntBitCount<BITS> as SupportedInt>::Uint>: From<Uint<BITS, LIMBS>>,
{
    #[inline]
    fn stv_to_tokens(&self) -> WordToken {
        let stv: Converted<<IntBitCount<BITS> as SupportedInt>::Uint> = (*self).into();
        SolTypeValue::<sol_data::Uint<BITS>>::stv_to_tokens(&stv.0)
    }

    #[inline]
    fn stv_abi_encode_packed_to(&self, out: &mut alloy_sol_types::private::Vec<u8>) {
        let stv: Converted<<IntBitCount<BITS> as SupportedInt>::Uint> = (*self).into();
        SolTypeValue::<sol_data::Uint<BITS>>::stv_abi_encode_packed_to(&stv.0, out)
    }

    #[inline]
    fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
        self.stv_to_tokens().0
    }
}

// Marker for built-in number types
trait Builtin {}

impl Builtin for i8 {}
impl Builtin for i16 {}
impl Builtin for i32 {}
impl Builtin for i64 {}
impl Builtin for i128 {}
impl Builtin for u8 {}
impl Builtin for u16 {}
impl Builtin for u32 {}
impl Builtin for u64 {}
impl Builtin for u128 {}

// Int or Uint that has been converted to the appropriate SupportedInt type
struct Converted<T>(T);

impl<T: Builtin, const BITS: usize, const LIMBS: usize> From<T> for Converted<Signed<BITS, LIMBS>>
where
    Signed<BITS, LIMBS>: TryFrom<T>,
    <Signed<BITS, LIMBS> as TryFrom<T>>::Error: core::fmt::Debug,
{
    fn from(int: T) -> Self {
        Converted(int.try_into().unwrap())
    }
}

impl<T: Builtin, const BITS: usize, const LIMBS: usize> From<Signed<BITS, LIMBS>> for Converted<T>
where
    IntBitCount<BITS>: SupportedInt<Int = T>,
    T: TryFrom<Signed<BITS, LIMBS>>,
    <T as TryFrom<Signed<BITS, LIMBS>>>::Error: core::fmt::Debug,
{
    fn from(int: Signed<BITS, LIMBS>) -> Self {
        Converted(int.try_into().unwrap())
    }
}

impl<const FB: usize, const FL: usize, const TB: usize, const TL: usize> From<Signed<FB, FL>>
    for Converted<Signed<TB, TL>>
{
    fn from(int: Signed<FB, FL>) -> Self {
        let slice = int.as_limbs();
        if slice.len() < TL {
            let mut limbs = [0; TL];
            limbs[..slice.len()].copy_from_slice(slice);
            Converted(Signed::from_limbs(limbs))
        } else {
            let (head, _tail) = slice.split_at(TL);
            let mut limbs = [0; TL];
            limbs.copy_from_slice(head);
            /* TODO overflow check needed?
            let mut overflow = tail.iter().any(|&limb| limb != 0);
            if TL > 0 {
                overflow |= limbs[TL - 1] > Signed::<TB, TL>::MASK;
                limbs[TL - 1] &= Signed::<TB, TL>::MASK;
            }
            */
            Converted(Signed::from_limbs(limbs))
        }
    }
}

impl<T: Builtin, const BITS: usize, const LIMBS: usize> From<Uint<BITS, LIMBS>> for Converted<T>
where
    IntBitCount<BITS>: SupportedInt<Uint = T>,
    T: TryFrom<Uint<BITS, LIMBS>>,
    <T as TryFrom<Uint<BITS, LIMBS>>>::Error: core::fmt::Debug,
{
    fn from(uint: Uint<BITS, LIMBS>) -> Self {
        Converted(uint.try_into().unwrap())
    }
}

impl<const BITS: usize, const LIMBS: usize> From<Uint<BITS, LIMBS>> for Converted<Uint<256, 4>> {
    fn from(uint: Uint<BITS, LIMBS>) -> Self {
        Converted(Uint::from_limbs_slice(uint.as_limbs()))
    }
}
