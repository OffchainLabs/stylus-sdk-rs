// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

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
