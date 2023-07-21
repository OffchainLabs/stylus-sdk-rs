// Copyright 2023, Offchain Labs, Inc.
// For license information, see https://github.com/OffchainLabs/nitro/blob/master/LICENSE

use ahash::RandomState;
use core::{cell::RefCell, default::Default};
use std::marker::PhantomData;
use hashbrown::{HashMap, HashSet};

use crate::{load_bytes32, store_bytes32, tx};

pub trait BackendStorage {
    fn new() -> Self;
    fn seed(&mut self) -> [u8; 32];
    fn load(&mut self, key: B256) -> B256;
    fn store(&mut self, key: B256, value: B256);
}

#[derive(Clone, Debug, Default)]
pub struct StylusBackendStorage{}

impl BackendStorage for StylusBackendStorage {
    fn new() -> Self {
        StylusBackendStorage::default()
    }

    fn seed(&mut self) -> [u8; 32] {
        let mut data = [0; 32];
        data[..20].copy_from_slice(&tx::origin().0);
        data
    }

    fn load(&mut self, key: B256) -> B256 {
        load_bytes32(key)
    }

    fn store(&mut self, key: B256, value: B256) {
        store_bytes32(key, value)
    }
}

#[derive(Clone, Debug, Default)]
pub struct MemoryBackendStorage{
    slots: HashMap<B256, B256>,
}

impl BackendStorage for MemoryBackendStorage {
    fn new() -> Self {
        MemoryBackendStorage::default()
    }

    fn seed(&mut self) -> [u8; 32] {
        [0; 32]
    }

    fn load(&mut self, key: B256) -> B256 {
        *self.slots.get(&key).unwrap_or(&B256::default())
    }

    fn store(&mut self, key: B256, value: B256) {
        if value.is_zero() {
            self.slots.remove(&key);
        } else {
            self.slots.insert(key, value);
        }
    }
}

#[derive(Clone, Debug)]
pub struct SubKey {
    key: B256,
    offset: usize,
    size: usize,
}

impl SubKey {
    pub fn new(key: B256, offset: usize, size: usize) -> Self {
        let offset_end = offset + size;
        if offset_end > 32 || offset > 32 || size > 32 {
            panic!("SubKey {key} is invalid, offset:{offset}, size:{size}")
        }
        Self {key, offset, size}
    }
}

#[derive(Debug)]
pub struct CachingStorage<Backend: BackendStorage> {
    backend: Backend,
    save_on_drop: bool,
    cached_slots: HashMap<B256, B256, RandomState>,
    dirty_slots: HashSet<B256, RandomState>,
    current_slot: usize,
    next_offset: usize,
}

impl<Backend: BackendStorage> CachingStorage<Backend> {
    pub fn new(mut backend: Backend) -> Self {
        let seed = backend.seed();
        let k0 = u64::from_be_bytes(seed[0..8].try_into().unwrap());
        let k1 = u64::from_be_bytes(seed[8..16].try_into().unwrap());
        let k2 = u64::from_be_bytes(seed[16..24].try_into().unwrap());
        let k3 = u64::from_be_bytes(seed[24..32].try_into().unwrap());
        Self {
            backend,
            save_on_drop: false,
            cached_slots: HashMap::with_hasher(RandomState::with_seeds(k0, k1, k2, k3)),
            dirty_slots: HashSet::with_hasher(RandomState::with_seeds(k0, k1, k2, k3)),
            current_slot: 0,
            next_offset: 0,
        }
    }

    pub fn save_on_drop(&mut self) {
        self.save_on_drop = true;
    }

    pub fn get_slot(&mut self, key: B256) -> B256 {
        *self.cached_slots.entry(key).or_insert(self.backend.load(key))
    }

    pub fn set_slot(&mut self, key: B256, data: B256) {
        if data != self.get_slot(key) {
            self.cached_slots.insert(key, data);
            self.dirty_slots.insert(key);
        }
    }

    pub fn new_sub_key(&mut self, size: usize) -> SubKey {
        if size > 32 {
            // TODO: handle large data
            panic!();
        }
        let mut new_offset = self.next_offset + size;
        if new_offset > 32 {
            self.current_slot += 1;
            self.next_offset = 0;
            new_offset = size;
        }
        let current_offset = self.next_offset;
        let current_slot = self.current_slot;
        self.next_offset = new_offset;
        SubKey {
            key: current_slot.into(),
            offset: current_offset,
            size,
        }
    }

    pub fn get<const S: usize>(&mut self, sub_key: &SubKey) -> [u8; S] {
        let data = self.get_slot(sub_key.key);
        data[sub_key.offset..sub_key.offset + sub_key.size].try_into().unwrap()
    }

    pub fn set<const S: usize>(&mut self, sub_key: &SubKey, data: &[u8; S]) {
        self.set_with_callback(sub_key, |dest: &mut [u8; S]| {
            let mut start = 0;
            if data.len() < sub_key.size {
                // Clear missing most significant bytes (big endian)
                start += sub_key.size - data.len();
                dest[0..start].fill(0);
            };
            dest[start..sub_key.size].copy_from_slice(data);
        })
    }

    pub fn set_with_callback<const S: usize, F: Fn(&mut [u8; S])>(&mut self, sub_key: &SubKey, setfn: F) {
        let mut slot = self.get_slot(sub_key.key);
        setfn(<&mut [u8; S]>::try_from(&mut slot[sub_key.offset..sub_key.offset + sub_key.size]).unwrap());
        self.set_slot(sub_key.key, slot)
    }
}

impl<Backend: BackendStorage> Drop for CachingStorage<Backend> {
    fn drop(&mut self) {
        if self.save_on_drop {
            for key in self.dirty_slots.iter() {
                if let Some(slot) = self.cached_slots.get(key) {
                    self.backend.store(*key, *slot);
                }
            }
        }
    }
}

pub trait StorageSerde<T, const S: usize> {
    fn deserialize(data: [u8; S]) -> T;
    fn serialize(&self, dest: &mut [u8; S]);
}

pub struct StorageBacked<'a, T, const S: usize, Backend: BackendStorage> {
    storage: &'a RefCell<CachingStorage<Backend>>,
    sub_key: SubKey,
    phantom: PhantomData<T>,
}

impl<'a, T: Copy+StorageSerde<T, S>, const S: usize, Storage: BackendStorage> StorageBacked<'a, T, S, Storage> {
    pub fn get(&self) -> T {
        T::deserialize(self.storage.borrow_mut().get(&self.sub_key))
    }
    pub fn set(&mut self, value: T) {
        self.storage.borrow_mut().set_with_callback(&self.sub_key, |dest: &mut [u8; S]| value.serialize(dest))
    }
}

pub struct Storage<Backend: BackendStorage> {
    pub cell: RefCell<CachingStorage<Backend>>,
}

impl<Backend: BackendStorage> Storage<Backend> {
    pub fn new(storage: CachingStorage<Backend>) -> Self {
        Self {
            cell: RefCell::new(storage),
        }
    }

    pub fn new_storage_backed<T: Copy+StorageSerde<T, S>, const S: usize>(&self) -> StorageBacked<T, S, Backend> {
        StorageBacked{
            storage: &self.cell,
            sub_key: self.cell.borrow_mut().new_sub_key(S),
            phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::B256;
    use super::*;

    #[test]
    fn test_slots() {
        let backend = MemoryBackendStorage::new();
        let mut cs = CachingStorage::new(backend);
        let key0: [u8; 32] = [0; 32];
        let mut key1: [u8; 32] = [0; 32];
        key1[31] = 1;
        let mut s0 = cs.get_slot(key0.into());
        let mut s1 = cs.get_slot(key1.into());
        s0.0[31] = 42;
        s1.0[31] = 43;
        cs.set_slot(key0.into(), s0);
        cs.set_slot(key1.into(), s1);
        let s0_new = cs.get_slot(key0.into());
        let s1_new = cs.get_slot(key1.into());
        assert_eq!(s0, s0_new);
        assert_eq!(s1, s1_new);
    }

    #[test]
    #[should_panic(expected = "SubKey 0000000000000000000000000000000000000000000000000000000000000000 is invalid, offset:33, size:0")]
    fn test_panic_large_offset() {
        let _ = SubKey::new(B256::default(), 33, 0);
    }

    #[test]
    #[should_panic(expected = "SubKey 0000000000000000000000000000000000000000000000000000000000000000 is invalid, offset:0, size:33")]
    fn test_panic_large_size() {
        let _ = SubKey::new(B256::default(), 0, 33);
    }

    #[test]
    #[should_panic(expected = "SubKey 0000000000000000000000000000000000000000000000000000000000000000 is invalid, offset:1, size:32")]
    fn test_panic_overflow() {
        let _ = SubKey::new(B256::default(), 1, 32);
    }

    #[test]
    fn test_new_zero_size() {
        let backend = MemoryBackendStorage::new();
        let mut cs = CachingStorage::new(backend);
        let k0 = SubKey::new(B256::default(), 32, 0);
        let s0bytes = [0; 0];
        cs.set(&k0, &s0bytes);
        assert_eq!(s0bytes, cs.get(&k0));
    }

    #[test]
    fn test_sub_keys() {
        let backend = MemoryBackendStorage::new();
        let mut cs = CachingStorage::new(backend);
        let k0 = cs.new_sub_key(8);
        let k1 = cs.new_sub_key(8);
        let k2 = cs.new_sub_key(16);
        assert_eq!(0, cs.current_slot);
        assert_eq!(32, cs.next_offset);
        let k3 = cs.new_sub_key(16);
        assert_eq!(1, cs.current_slot);
        assert_eq!(16, cs.next_offset);

        let s0: u64 = 45;
        let s1: u64 = 46;
        let s2: u128 = 47;
        let s3: u128 = 48;
        cs.set(&k0, &s0.to_be_bytes());
        cs.set(&k1, &s1.to_be_bytes());
        cs.set(&k2, &s2.to_be_bytes());
        cs.set(&k3, &s3.to_be_bytes());
        assert_eq!(s0, u64::from_be_bytes(cs.get(&k0)));
        assert_eq!(s1, u64::from_be_bytes(cs.get(&k1)));
        assert_eq!(s2, u128::from_be_bytes(cs.get(&k2)));
        assert_eq!(s3, u128::from_be_bytes(cs.get(&k3)));
    }

    #[test]
    fn test_storage_backed() {
        #[derive(Clone, Copy)]
        struct TestingCustomType {
            foo: u64,
            bar: u64,
        }

        impl StorageSerde<TestingCustomType, 16> for TestingCustomType {
            fn deserialize(data: [u8; 16]) -> Self {
                Self{
                    foo: u64::from_be_bytes(data[..8].try_into().unwrap()),
                    bar: u64::from_be_bytes(data[8..16].try_into().unwrap()),
                }
            }

            fn serialize(&self, dest: &mut [u8; 16]) {
                dest[..8].copy_from_slice(&self.foo.to_be_bytes());
                dest[8..16].copy_from_slice(&self.bar.to_be_bytes());
            }
        }

        let backend = MemoryBackendStorage::new();
        let cs = CachingStorage::new(backend);
        let storage = Storage::new(cs);
        let mut k0 = storage.new_storage_backed::<TestingCustomType, 16>();
        let mut k1 = storage.new_storage_backed::<TestingCustomType, 16>();
        let mut s0 = k0.get();
        let mut s1 = k1.get();
        s0.foo = 12;
        s0.bar = 13;
        s1.foo = 14;
        s1.bar = 15;
        k0.set(s0);
        k1.set(s1);
        assert_eq!(s0.foo, k0.get().foo);
        assert_eq!(s0.bar, k0.get().bar);
        assert_eq!(s1.foo, k1.get().foo);
        assert_eq!(s1.bar, k1.get().bar);
    }
}
