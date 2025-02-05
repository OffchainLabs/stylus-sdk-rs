// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use crate::{
    abi::{AbiType, ConstString},
    util::evm_padded_length,
};
use alloc::vec::Vec;
use alloy_sol_types::{abi::token::PackedSeqToken, private::SolTypeValue, SolType, SolValue};
use core::ops::{Deref, DerefMut};

/// Represents a [`bytes`] in Solidity.
///
/// [`bytes`]: https://docs.soliditylang.org/en/latest/types.html#bytes-and-string-as-arrays
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Bytes(pub Vec<u8>);

impl From<Bytes> for Vec<u8> {
    fn from(value: Bytes) -> Self {
        value.0
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(b: Vec<u8>) -> Self {
        Self(b)
    }
}

impl From<alloy_primitives::Bytes> for Bytes {
    fn from(value: alloy_primitives::Bytes) -> Self {
        Self(value.to_vec())
    }
}

impl Deref for Bytes {
    type Target = Vec<u8>;

    fn deref(&self) -> &Vec<u8> {
        &self.0
    }
}

impl DerefMut for Bytes {
    fn deref_mut(&mut self) -> &mut Vec<u8> {
        &mut self.0
    }
}

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsMut<[u8]> for Bytes {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Provides a corresponding [`SolType`] for an [`abi`] [`Bytes`].
///
/// [`abi`]: crate::abi
pub struct BytesSolType;

impl SolTypeValue<Self> for Bytes {
    #[inline]
    fn stv_to_tokens(&self) -> <Self as alloy_sol_types::SolType>::Token<'_> {
        self.0.tokenize()
    }

    #[inline]
    fn stv_abi_encoded_size(&self) -> usize {
        32 + evm_padded_length(self.len())
    }

    #[inline]
    fn stv_abi_packed_encoded_size(&self) -> usize {
        32 + evm_padded_length(self.len())
    }

    #[inline]
    fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
        self.0.eip712_data_word()
    }

    #[inline]
    fn stv_abi_encode_packed_to(&self, out: &mut alloy_sol_types::private::Vec<u8>) {
        self.0.abi_encode_packed_to(out)
    }
}

impl SolType for Bytes {
    type RustType = Bytes;

    type Token<'a> = PackedSeqToken<'a>;

    const ENCODED_SIZE: Option<usize> = None;
    const PACKED_ENCODED_SIZE: Option<usize> = None;

    const SOL_NAME: &'static str = "bytes";

    #[inline]
    fn valid_token(_: &Self::Token<'_>) -> bool {
        true // Any PackedSeqToken is valid bytes
    }

    #[inline]
    fn detokenize(token: Self::Token<'_>) -> Self::RustType {
        Bytes(token.0.into())
    }
}

impl SolValue for Bytes {
    type SolType = Self;
}

impl AbiType for Bytes {
    type SolType = Self;

    const ABI: ConstString = ConstString::new("bytes");

    const EXPORT_ABI_ARG: ConstString = Self::ABI.concat(ConstString::new(" calldata"));

    const EXPORT_ABI_RET: ConstString = Self::ABI.concat(ConstString::new(" memory"));
}

#[cfg(test)]
mod tests {
    use alloy_primitives::hex;

    use super::*;
    use crate::abi;

    #[test]
    fn bytes_sol_type() {
        assert_eq!(Bytes::ABI.as_str(), "bytes");
        assert_eq!(Bytes::EXPORT_ABI_ARG.as_str(), "bytes calldata");
        assert_eq!(Bytes::EXPORT_ABI_RET.as_str(), "bytes memory");
    }

    #[test]
    fn bytes_abi() {
        assert_eq!(Bytes::SOL_NAME, "bytes");
        assert_eq!(Bytes::ENCODED_SIZE, None);
        if !Bytes::DYNAMIC {
            panic!();
        }
        assert_eq!(
            <Bytes as SolType>::abi_encoded_size(&Bytes(vec![1, 2, 3, 4])),
            64
        );
    }

    #[test]
    fn encode_decode_empty_bytes() {
        abi::test_encode_decode_params(
            (Bytes(vec![]),),
            hex!(
                "0000000000000000000000000000000000000000000000000000000000000020"
                "0000000000000000000000000000000000000000000000000000000000000000"
            ),
        );
    }

    #[test]
    fn encode_decode_one_byte() {
        abi::test_encode_decode_params(
            (Bytes(vec![100]),),
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
        let value = (Bytes(input),);
        let encoded = hex!(
            "0000000000000000000000000000000000000000000000000000000000000020"
            "0000000000000000000000000000000000000000000000000000000000000028"
            "0102030400000000000000000000000000000000000000000000000000000000"
            "0000000005060708000000000000000000000000000000000000000000000000"
        );
        abi::test_encode_decode_params(value, encoded);
    }

    #[test]
    fn encode_decode_bytes_tuple() {
        let mut input = Vec::with_capacity(40);
        input.extend([1, 2, 3, 4]);
        input.extend([0u8; 32]);
        input.extend([5, 6, 7, 8]);
        let value = (Bytes(input), Bytes(vec![]), Bytes(vec![1, 2, 3, 4]));

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

        abi::test_encode_decode_params(value, encoded)
    }
}
