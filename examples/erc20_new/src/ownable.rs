use stylus_sdk::{alloy_primitives::Address, msg, prelude::*, storage::StorageAddress};

#[solidity_storage]
pub struct Ownable {
    owner: StorageAddress,
}

impl Ownable {
    pub fn owner(&self) -> Address {
        self.owner.get()
    }
    pub fn is_owner(&self) -> bool {
        let current_owner = self.owner.get();
        current_owner == msg::sender()
    }
    pub fn transfer_ownership(&mut self, new_owner: Address) -> bool {
        if !self.is_owner() || new_owner == Address::ZERO {
            return false;
        }

        self.owner.set(new_owner);
        true
    }
    pub fn renounce_ownership(&mut self) -> bool {
        if !self.is_owner() {
            return false;
        }

        self.owner.set(Address::ZERO);
        true
    }
}

pub trait IOwnable {
    fn owner(&self) -> Address;
    fn transfer_ownership(&mut self, new_owner: Address) -> bool;
    fn renounce_ownership(&mut self) -> bool;
}
