// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

//! Support for generic integer types found in [alloy_primitives].

use alloy_primitives::{Signed, Uint};
use alloy_sol_types::{
    sol_data::{self, IntBitCount, SupportedInt},
    SolType,
};

use super::{AbiType, ConstString};

impl<const BITS: usize, const LIMBS: usize> AbiType for Signed<BITS, LIMBS>
where IntBitCount<BITS>: SupportedInt<Int = Self>,
{
    type SolType = sol_data::Int<BITS>;

    const ABI: ConstString = ConstString::new(Self::SolType::SOL_NAME);
}

impl<const BITS: usize, const LIMBS: usize> AbiType for Uint<BITS, LIMBS>
where IntBitCount<BITS>: SupportedInt<Uint = Self>,
{
    type SolType = sol_data::Uint<BITS>;

    const ABI: ConstString = ConstString::new(Self::SolType::SOL_NAME);
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
