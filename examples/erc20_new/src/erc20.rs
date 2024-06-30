use stylus_sdk::{
    alloy_primitives::{Address, U256},
    prelude::*,
    storage::{StorageAddress, StorageMap, StorageString, StorageU256, StorageU8},
};

#[solidity_storage]
pub struct ERC20 {
    balances: StorageMap<Address, StorageU256>,
    total_supply: StorageU256,
}

impl ERC20 {
    pub fn balance_of(&self, account: Address) -> U256 {
        self.balances.get(account)
    }

    pub fn total_supply(&self) -> U256 {
        self.total_supply.get()
    }

    pub fn transfer_from(&mut self, sender: Address, recipient: Address, amount: U256) -> bool {
        let current_sender_balance = self.balance_of(sender);
        if current_sender_balance < amount {
            return false;
        }
        let current_recipient_balance = self.balance_of(recipient);

        let future_sender_balance = current_sender_balance - amount;
        let future_recipient_balance = current_recipient_balance + amount;

        self.balances.insert(sender, future_sender_balance);
        self.balances.insert(recipient, future_recipient_balance);

        true
    }

    pub fn mint(&mut self, recipient: Address, amount: U256) {
        let current_supply = self.total_supply.get();
        let future_supply = current_supply + amount;

        let current_balance = self.balances.get(recipient);
        let future_balance = current_balance + amount;

        self.total_supply.set(future_supply);
        self.balances.insert(recipient, future_balance);
    }
}

pub trait IERC20 {
    fn name(&self) -> String;
    fn symbol(&self) -> String;
    fn decimals(&self) -> U256;
    fn total_supply(&self) -> U256;
    fn balance_of(&self, account: Address) -> U256;
    fn transfer(&mut self, to: Address, value: U256) -> bool;
    // fn transfer_from(&mut self, from: Address, to: Address, value: U256) -> bool;
    // fn approve(&mut self, spender: Address, value: U256) -> bool;
    // fn allowance(&self, owner: Address, spender: Address) -> U256;
}
