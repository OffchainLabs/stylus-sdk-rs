// Copyright 2023, Offchain Labs, Inc.
// For license information, see https://github.com/OffchainLabs/nitro/blob/master/LICENSE

/*use std::ops::{Deref, DerefMut};

use alloy_primitives::{FixedBytes, B256};
use fnv::FnvHashMap as HashMap;
use lazy_static::lazy_static;

use crate::{load_bytes32, store_bytes32};

pub type StorageOffset = FixedBytes<33>;

pub struct StorageCache(HashMap<B256, StorageWord>);

impl Deref for StorageCache {
    type Target = HashMap<B256, StorageWord>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StorageCache {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

struct StorageWord {
    /// The current value of the slot
    value: B256,
    /// The value in the EVM state trie, if known
    known: Option<B256>,
}

impl StorageWord {
    fn new(known: B256) -> Self {
        Self {
            value: known,
            known: Some(known),
        }
    }
}

lazy_static! {
    /// Global cache managing permanent storage operations
    static ref CACHE: StorageCache = StorageCache(HashMap::default());
}*/

/*impl StorageCache {
    /// Retrieves `N ≤ 32` bytes from permanent storage, performing [`SLOAD`]'s only as needed.
    /// Note that the bytes must exist in a single, 32-byte EVM word.
    ///
    /// [`SLOAD`]: https://www.evm.codes/#54
    pub fn get<const Offset: usize, const N: usize>(key: B256) -> FixedBytes<N> {
        let value = CACHE
            .entry(key)
            .or_insert_with(|| StorageWord::new(load_bytes32(key)))
            .value;

        unsafe {
            // TODO: use `last_chunk::<Offset>()` when stable
            let (_, value) = value.split_at(Offset);
            FixedBytes::from_slice(value)
        }
    }

    /*pub fn store<const N: usize>(key: B256, value: FixedBytes<N>) {
        if let Some(word) = CACHE.get(&key) {
            unsafe { std::ptr::copy(value.as_ptr(), word.value[32 - N..].as_mut_ptr(), N) };
            return;
        }

        if N == 32 {
            CACHE.insert(
                key,
                StorageWord {
                    value: B256::try_from(value),
                    known: None,
                },
            );
            return;
        }

        let known = load_bytes32(key);
        //let
    }*/

    pub fn flush() {
        for (key, entry) in &mut CACHE.0 {
            if Some(entry.value) != entry.known {
                store_bytes32(*key, entry.value);
            }
        }
    }
}
 */

use alloy_primitives::{aliases::B32, FixedBytes, B256, U256};

use crate::{crypto, hostio::native_keccak256};

// TODO: use const generics once stable to elide runtime keccaks
pub trait InitStorage {
    const SIZE: u8 = 32;

    fn init(slot: U256, offset: u8) -> Self;
}

/// Address accessor
pub struct AddressAcc {
    slot: U256,
    offset: u8,
}

impl InitStorage for AddressAcc {
    const SIZE: u8 = 20;

    fn init(slot: U256, offset: u8) -> Self {
        Self { slot, offset }
    }
}

pub struct BlockHashAcc {
    slot: U256,
    offset: u8,
}

impl InitStorage for BlockHashAcc {
    fn init(slot: U256, offset: u8) -> Self {
        Self { slot, offset }
    }
}

pub struct StorageArray {
    slot: U256,
    base: B256,
}

impl InitStorage for StorageArray {
    fn init(slot: U256, offset: u8) -> Self {
        debug_assert!(offset == 0);
        let base = crypto::keccak(&slot.to_be_bytes::<32>());
        Self { slot, base }
    }
}

impl StorageArray {
    fn len() -> usize {
        todo!()
    }
}
