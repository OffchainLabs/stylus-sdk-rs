// Copyright 2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use super::{AbiType, ConstString};
use alloc::borrow::Cow;
use alloy_primitives::FixedBytes;
use alloy_sol_types::{
    sol_data::{ByteCount, SupportedFixedBytes},
    token::WordToken,
    Encodable, SolType, Word,
};

pub struct FixedBytesSolType<const N: usize>;

impl<const N: usize> FixedBytesSolType<N> {
    const SOL_TYPE_NAME: ConstString =
        ConstString::new("bytes").concat(ConstString::from_decimal_number(N));

    fn to_word(bytes: &FixedBytes<N>) -> Word {
        let mut out = Word::ZERO;
        out[..N].copy_from_slice(&bytes.0);
        out
    }
}

impl<const N: usize> SolType for FixedBytesSolType<N>
where
    ByteCount<N>: SupportedFixedBytes,
{
    type RustType = FixedBytes<N>;

    type TokenType<'a> = WordToken;

    fn sol_type_name() -> Cow<'static, str> {
        Self::SOL_TYPE_NAME.as_str().into()
    }

    fn type_check(token: &Self::TokenType<'_>) -> alloy_sol_types::Result<()> {
        // Fail if any padding bytes are non-zero
        if token.0[N..].iter().any(|b| *b != 0) {
            return Err(Self::type_check_fail(token.as_slice()));
        }
        Ok(())
    }

    fn detokenize(token: Self::TokenType<'_>) -> Self::RustType {
        let mut out = FixedBytes([0u8; N]);
        out.copy_from_slice(&token.0[..N]);
        out
    }

    fn eip712_data_word(bytes: &Self::RustType) -> alloy_sol_types::Word {
        // Fixed sized values in EIP712 are padded to a word
        Self::to_word(bytes)
    }

    fn encode_packed_to(bytes: &Self::RustType, out: &mut Vec<u8>) {
        // Packed encoding doesn't do any padding
        out.extend_from_slice(&bytes.0);
    }
}

impl<const N: usize> Encodable<FixedBytesSolType<N>> for FixedBytes<N>
where
    ByteCount<N>: SupportedFixedBytes,
{
    fn to_tokens(&self) -> WordToken {
        FixedBytesSolType::to_word(self).into()
    }
}

impl<const N: usize> AbiType for FixedBytes<N>
where
    ByteCount<N>: SupportedFixedBytes,
{
    type SolType = FixedBytesSolType<N>;

    const ABI: ConstString = ConstString::new("bytes").concat(ConstString::from_decimal_number(N));
}
