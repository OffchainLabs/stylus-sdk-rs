#![cfg_attr(not(feature = "export-abi"), no_main, no_std)]
extern crate alloc;

use crate::erc20::{Erc20, Erc20Params};
use alloc::{string::String, vec::Vec};
use stylus_sdk::{alloy_primitives::U256, call, msg, prelude::*};

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

mod erc20;

struct WethParams;

/// Immutable definitions
impl Erc20Params for WethParams {
    const NAME: &'static str = "Wrapped Ether Example";
    const SYMBOL: &'static str = "WETH";
    const DECIMALS: u8 = 18;
}

// The contract
sol_storage! {
    #[entrypoint] // Makes Weth the entrypoint
    struct Weth {
        #[borrow] // Allows erc20 to access Weth's storage and make calls
        Erc20<WethParams> erc20;
    }
}

// Another contract we'd like to call
sol_interface! {
    interface IMath {
        function sum(uint256[] values) pure returns (string, uint256);
    }
}

#[external]
#[inherit(Erc20<WethParams>)]
impl Weth {
    #[payable]
    pub fn deposit(&mut self) -> Result<(), Vec<u8>> {
        self.erc20.mint(msg::sender(), msg::value());
        Ok(())
    }

    pub fn withdraw(&mut self, amount: U256) -> Result<(), Vec<u8>> {
        self.erc20.burn(msg::sender(), amount)?;

        // send the user their funds
        call::transfer_eth(msg::sender(), amount)
    }

    // sums numbers
    pub fn sum(values: Vec<U256>) -> Result<(String, U256), Vec<u8>> {
        Ok(("sum".into(), values.iter().sum()))
    }

    // calls the sum() method from the interface
    pub fn sum_with_helper(&self, helper: IMath, values: Vec<U256>) -> Result<U256, Vec<u8>> {
        let (text, sum) = helper.sum(self, values)?;
        assert_eq!(&text, "sum");
        Ok(sum)
    }
}
