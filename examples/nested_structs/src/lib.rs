// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(feature = "contract-client-gen", allow(unused_imports))]

extern crate alloc;

use alloy_primitives::Address;
use alloy_sol_types::sol;
use stylus_sdk::{
    prelude::*,
    storage::{StorageAddress, StorageMap, StorageString, StorageVec},
};

sol! {
    #[derive(Debug, AbiType)]
    struct User {
        address address;
        string name;
        Dog[] dogs;
    }

    #[derive(Debug, AbiType)]
    struct Dog {
        string name;
        string breed;
    }

    #[derive(Debug)]
    error NotFound();

    #[derive(Debug)]
    error AlreadyExists();

    #[derive(Debug)]
    error InvalidParam();
}

#[derive(SolidityError, Debug)]
pub enum NestedStructsErrors {
    NotFound(NotFound),
    AlreadyExists(AlreadyExists),
    InvalidParam(InvalidParam),
}

#[storage]
struct StorageDog {
    name: StorageString,
    breed: StorageString,
}

#[storage]
struct StorageUser {
    name: StorageString,
    dogs: StorageVec<StorageDog>,
}

#[storage]
#[entrypoint]
struct NestedStructs {
    // Store the user data in a map
    user_data: StorageMap<Address, StorageUser>,

    // Create a list of addresses because we can't iterate over the map
    user_list: StorageVec<StorageAddress>,
}

#[public]
impl NestedStructs {
    pub fn add_user(&mut self, address: Address, name: String) -> Result<(), NestedStructsErrors> {
        if name.is_empty() {
            return Err(InvalidParam {}.into());
        }
        let entry = self.user_data.get(address);
        if !entry.name.is_empty() {
            return Err(AlreadyExists {}.into());
        }
        self.user_list.push(address);
        self.user_data.setter(address).name.set_str(name);
        Ok(())
    }

    pub fn add_dogs(&mut self, user: Address, dogs: Vec<Dog>) -> Result<(), NestedStructsErrors> {
        for dog in dogs.iter() {
            if dog.name.is_empty() || dog.breed.is_empty() {
                return Err(InvalidParam {}.into());
            }
        }
        let entry = self.user_data.get(user);
        if entry.name.is_empty() {
            return Err(NotFound {}.into());
        }
        let mut user_setter = self.user_data.setter(user);
        for dog in dogs {
            let mut dog_setter = user_setter.dogs.grow();
            dog_setter.name.set_str(dog.name);
            dog_setter.breed.set_str(dog.breed);
        }
        Ok(())
    }

    pub fn get_user(&self, address: Address) -> Result<User, NestedStructsErrors> {
        let entry = self.user_data.get(address);
        let name = entry.name.get_string();
        if name.is_empty() {
            return Err(NotFound {}.into());
        }
        let dogs_len = entry.dogs.len();
        let mut dogs = Vec::with_capacity(dogs_len);
        for i in 0..dogs_len {
            // SAFETY: unwrap is safe because we got the len beforehand
            let dog_entry = entry.dogs.get(i).unwrap();
            let dog = Dog {
                name: dog_entry.name.get_string(),
                breed: dog_entry.breed.get_string(),
            };
            dogs.push(dog);
        }
        Ok(User {
            address,
            name,
            dogs,
        })
    }

    pub fn get_all_users(&self) -> Vec<User> {
        let users_len = self.user_list.len();
        let mut users = Vec::with_capacity(users_len);
        for i in 0..users_len {
            // SAFETY: unwrap is safe because we got the len beforehand
            let address = self.user_list.get(i).unwrap();
            // SAFETY: unwrap is safe because we know that every user in the list has a data
            let user = self.get_user(address).unwrap();
            users.push(user);
        }
        users
    }
}
