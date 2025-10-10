// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(feature = "contract-client-gen", allow(unused_imports))]

extern crate alloc;

use stylus_sdk::{
    alloy_primitives::{Address, U256},
    prelude::*,
    storage::{StorageAddress, StorageMap, StorageU256},
};

// ──────────────────────────────────────────────────────────────────────────────
// Traits (interfaces)
// ──────────────────────────────────────────────────────────────────────────────

#[public]
trait IErc20 {
    fn name(&self) -> String;
    fn symbol(&self) -> String;
    fn decimals(&self) -> U256;
    fn total_supply(&self) -> U256;
    fn balance_of(&self, account: Address) -> U256;
    fn transfer(&mut self, to: Address, value: U256) -> bool;
}

#[public]
trait IOwnable {
    fn owner(&self) -> Address;
    fn transfer_ownership(&mut self, new_owner: Address) -> bool;
    fn renounce_ownership(&mut self) -> bool;
}

// Extra trait with a “name-like” concept we also want to export.
// We'll give it a distinct Solidity-visible selector to avoid clobbering ERC-20's `name()`.
#[public]
trait IBranding {
    fn brand_name(&self) -> String;
}

// ──────────────────────────────────────────────────────────────────────────────
/* Storage components */
// ──────────────────────────────────────────────────────────────────────────────

#[storage]
struct Erc20 {
    balances: StorageMap<Address, StorageU256>,
    total_supply: StorageU256,
}

#[storage]
struct Ownable {
    owner: StorageAddress,
}

// ──────────────────────────────────────────────────────────────────────────────
/* Entrypoint contract */
// ──────────────────────────────────────────────────────────────────────────────

#[storage]
#[entrypoint]
struct Contract {
    erc20: Erc20,
    ownable: Ownable,
}

// One (and only one) public inherent impl with the router wiring.
// Add traits here to export them in the ABI.
#[public]
#[implements(IErc20, IOwnable, IBranding)]
impl Contract {}

// ──────────────────────────────────────────────────────────────────────────────
/* Trait implementations */
// ──────────────────────────────────────────────────────────────────────────────

#[public]
impl IErc20 for Contract {
    fn name(&self) -> String {
        "MyToken".to_string()
    }

    fn symbol(&self) -> String {
        "MTK".to_string()
    }

    fn decimals(&self) -> U256 {
        U256::from(18)
    }

    fn total_supply(&self) -> U256 {
        self.erc20.total_supply.get()
    }

    fn balance_of(&self, account: Address) -> U256 {
        self.erc20.balances.get(account)
    }

    fn transfer(&mut self, to: Address, value: U256) -> bool {
        // Example-only: fill in real checks/moves as needed
        let from = self.vm().msg_sender();
        if from == Address::ZERO || to == Address::ZERO {
            return false;
        }
        let from_bal = self.erc20.balances.get(from);
        if from_bal < value {
            return false;
        }
        self.erc20.balances.setter(from).set(from_bal - value);
        let to_bal = self.erc20.balances.get(to);
        self.erc20.balances.setter(to).set(to_bal + value);
        true
    }
}

#[public]
impl IOwnable for Contract {
    fn owner(&self) -> Address {
        self.ownable.owner.get()
    }

    fn transfer_ownership(&mut self, new_owner: Address) -> bool {
        let caller = self.vm().msg_sender();
        if caller != self.ownable.owner.get() || new_owner == Address::ZERO {
            return false;
        }
        self.ownable.owner.set(new_owner);
        true
    }

    fn renounce_ownership(&mut self) -> bool {
        let caller = self.vm().msg_sender();
        if caller != self.ownable.owner.get() {
            return false;
        }
        self.ownable.owner.set(Address::ZERO);
        true
    }
}

// Important part: give the extra name-like method a DISTINCT selector.
// This avoids colliding with ERC-20's `name()` in the ABI.
#[public]
impl IBranding for Contract {
    #[selector(name = "displayName")]
    fn brand_name(&self) -> String {
        "MyToken".to_string()
    }
}
