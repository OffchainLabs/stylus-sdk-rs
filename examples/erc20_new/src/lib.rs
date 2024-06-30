// Only run this as a WASM if the export-abi feature is not set.
#![cfg_attr(not(any(feature = "export-abi", test)), no_main)]
extern crate alloc;

// Modules and imports
pub mod erc20;
pub mod ownable;

use crate::erc20::*;
use crate::ownable::*;
use alloy_primitives::{Address, U256};
use stylus_sdk::{msg, prelude::*};

/// Initializes a custom, global allocator for Rust programs compiled to WASM.
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

/** CONSTANTS */
const NAME: &str = "MyToken";
const SYMBOL: &str = "MT";
const DECIMALS: u8 = 18;

#[solidity_storage]
#[entrypoint]
struct MyToken {
    erc20: ERC20,
    ownable: Ownable,
}

#[external]
impl IERC20 for MyToken {
    fn name(&self) -> String {
        NAME.into()
    }
    fn symbol(&self) -> String {
        SYMBOL.into()
    }
    fn decimals(&self) -> U256 {
        U256::from(DECIMALS)
    }
    fn total_supply(&self) -> U256 {
        self.erc20.total_supply()
    }
    fn balance_of(&self, account: Address) -> U256 {
        self.erc20.balance_of(account)
    }
    fn transfer(&mut self, to: Address, value: U256) -> bool {
        self.erc20.transfer_from(msg::sender(), to, value)
    }
}

// #[external]
impl IOwnable for MyToken {
    fn owner(&self) -> Address {
        self.ownable.owner()
    }
    fn renounce_ownership(&mut self) -> bool {
        self.ownable.renounce_ownership()
    }
    fn transfer_ownership(&mut self, new_owner: Address) -> bool {
        self.ownable.transfer_ownership(new_owner)
    }
}

// #[external]
// Developer-defined external methods
impl MyToken {
    // here the developer-defined mint method (which is not part of the
    // IERC20 API spec), uses internal methods from Ownable and ERC20 to
    // validate ownership before minting new supply
    fn mint(&mut self, recipient: Address, amount: U256) -> bool {
        if !self.ownable.is_owner() {
            return false;
        }

        self.erc20.mint(recipient, amount);
        true
    }
}
