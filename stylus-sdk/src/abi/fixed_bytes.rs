// Copyright 2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use super::{AbiType, ConstString};
use alloy_primitives::FixedBytes;
use alloy_sol_types::{
    abi::token::WordToken,
    private::SolTypeValue,
    sol_data::{ByteCount, SupportedFixedBytes},
    SolType, SolValue, Word,
};

/// SolType for FixedBytes.
pub struct FixedBytesSolType<const N: usize>;

impl<const N: usize> SolTypeValue<FixedBytesSolType<N>> for FixedBytes<N>
where
    ByteCount<N>: SupportedFixedBytes,
{
    #[inline]
    fn stv_to_tokens(&self) -> <FixedBytesSolType<N> as alloy_sol_types::SolType>::Token<'_> {
        let mut word = Word::ZERO;
        word[..N].copy_from_slice(&self.0);
        word.into()
    }

    #[inline]
    fn stv_abi_encoded_size(&self) -> usize {
        self.0.abi_encoded_size()
    }

    #[inline]
    fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
        SolTypeValue::<FixedBytesSolType<N>>::stv_to_tokens(self).0
    }

    #[inline]
    fn stv_abi_encode_packed_to(&self, out: &mut alloy_sol_types::private::Vec<u8>) {
        out.extend_from_slice(&self.0)
    }
}

impl<const N: usize> SolType for FixedBytesSolType<N>
where
    ByteCount<N>: SupportedFixedBytes,
{
    type RustType = FixedBytes<N>;

    type Token<'a> = WordToken;

    const ENCODED_SIZE: Option<usize> = Some(1);

    const SOL_NAME: &'static str = <ByteCount<N>>::NAME;

    fn valid_token(token: &Self::Token<'_>) -> bool {
        // Valid if all padding bytes are 0
        token.0[N..].iter().all(|b| *b == 0)
    }

    fn detokenize(token: Self::Token<'_>) -> Self::RustType {
        let mut out = FixedBytes([0u8; N]);
        out.copy_from_slice(&token.0[..N]);
        out
    }
}

// impl<const N: usize> SolValue for FixedBytes<N>
// where
//     ByteCount<N>: SupportedFixedBytes,
// {
//     type SolType = Self;
// }

impl<const N: usize> AbiType for FixedBytes<N>
where
    ByteCount<N>: SupportedFixedBytes,
{
    type SolType = FixedBytesSolType<N>;

    const ABI: ConstString = ConstString::new("bytes").concat(ConstString::from_decimal_number(N));
}

// XXX
// impl<const N: usize> Encodable<FixedBytesSolType<N>> for FixedBytes<N>
// where
//     ByteCount<N>: SupportedFixedBytes,
// {
//     fn to_tokens(&self) -> WordToken {
//         FixedBytesSolType::to_word(self).into()
//     }
// }
