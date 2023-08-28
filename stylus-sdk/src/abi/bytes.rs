// Copyright 2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use crate::{
    abi::{AbiType, ConstString},
    crypto,
    util::evm_padded_length,
};
use alloc::borrow::Cow;
use alloy_sol_types::{token::PackedSeqToken, Encodable, SolType};
use core::ops::{Deref, DerefMut};

/// Represents a [`bytes`] in Solidity.
///
/// [`bytes`]: https://docs.soliditylang.org/en/v0.8.21/types.html#bytes-and-string-as-arrays
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

pub struct BytesSolType;

impl SolType for BytesSolType {
    type RustType = Bytes;

    type TokenType<'a> = PackedSeqToken<'a>;

    const ENCODED_SIZE: Option<usize> = None;

    #[inline]
    fn encoded_size(bytes: &Self::RustType) -> usize {
        32 + evm_padded_length(bytes.len())
    }

    #[inline]
    fn sol_type_name() -> Cow<'static, str> {
        "bytes".into()
    }

    #[inline]
    fn type_check(_: &Self::TokenType<'_>) -> alloy_sol_types::Result<()> {
        Ok(()) // Any PackedSeqToken is valid bytes
    }

    #[inline]
    fn detokenize(token: Self::TokenType<'_>) -> Self::RustType {
        Bytes(token.0.into())
    }

    #[inline]
    fn eip712_data_word(bytes: &Self::RustType) -> alloy_sol_types::Word {
        // "The dynamic values bytes and string are encoded as a keccak256 hash of their contents."
        // - https://eips.ethereum.org/EIPS/eip-712#definition-of-encodedata
        crypto::keccak(bytes)
    }

    #[inline]
    fn encode_packed_to(bytes: &Self::RustType, out: &mut Vec<u8>) {
        out.extend_from_slice(bytes);
    }
}

impl Encodable<BytesSolType> for Bytes {
    fn to_tokens(&self) -> PackedSeqToken<'_> {
        PackedSeqToken(&self.0)
    }
}

impl AbiType for Bytes {
    type SolType = BytesSolType;

    const ABI: ConstString = ConstString::new("bytes");

    const EXPORT_ABI_ARG: ConstString = Self::ABI.concat(ConstString::new(" calldata"));

    const EXPORT_ABI_RET: ConstString = Self::ABI.concat(ConstString::new(" memory"));
}
