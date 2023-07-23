// Copyright 2022-2023, Offchain Labs, Inc.
// For license information, see https://github.com/OffchainLabs/nitro/blob/master/LICENSE

use alloy_primitives::B256;

pub fn keccak<T: AsRef<[u8]>>(bytes: T) -> B256 {
    alloy_primitives::keccak256(bytes)
}
