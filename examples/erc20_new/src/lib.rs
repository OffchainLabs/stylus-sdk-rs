// Only run this as a WASM if the export-abi feature is not set.
#![cfg_attr(not(any(feature = "export-abi", test)), no_main)]
extern crate alloc;

// Modules and imports
mod erc20;

use crate::erc20::*;
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

/**
 * Core problem with current SDK for Rust-style composition pattern
 * is the inability to define multiple #[external] impl blocks. If so,
 * then I could seamlessly compose IERC20 or any number of traits to
 * define my contract's public API.
 *
 * Part of the reason this is difficult is that macros only have visibility
 * into a single block. We cannot combine multiple #[external] impls into 1.
 * There are patterns around this, but will require refactoring the SDK's
 * router.
 */

// #[external]
impl MyToken {
    fn _mint(&mut self, recipient: Address, amount: U256) {
        self.erc20.mint(recipient, amount)
    }
}
