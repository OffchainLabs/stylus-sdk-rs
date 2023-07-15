// Copyright 2023, Offchain Labs, Inc.
// For license information, see https://github.com/OffchainLabs/nitro/blob/master/LICENSE

use crate::hostio::{self, wrap_hostio};
use alloy_primitives::{Address, B256, U64};

/// Gets the price of ink in evm gas basis points. See [`Ink and Gas`] for more information on
/// Stylus's compute-pricing model.
///
/// [`Ink and Gas`]: https://developer.arbitrum.io/TODO
pub fn ink_price() -> U64 {
    unsafe { hostio::CACHED_INK_PRICE.get().try_into().unwrap() }
}

/// Converts evm gas to ink. See [`Ink and Gas`] for more information on
/// Stylus's compute-pricing model.
///
/// [`Ink and Gas`]: https://developer.arbitrum.io/TODO
#[allow(clippy::inconsistent_digit_grouping)]
pub fn gas_to_ink(gas: U64) -> U64 {
    gas.saturating_mul(U64::try_from(100_00).unwrap()) / ink_price()
}

/// Converts ink to evm gas. See [`Ink and Gas`] for more information on
/// Stylus's compute-pricing model.
///
/// [`Ink and Gas`]: https://developer.arbitrum.io/TODO
#[allow(clippy::inconsistent_digit_grouping)]
pub fn ink_to_gas(ink: U64) -> U64 {
    ink.saturating_mul(ink_price()) / U64::try_from(100_00).unwrap()
}

wrap_hostio!(
    /// Gets the gas price in wei per gas, which on Arbitrum chains equals the basefee.
    gas_price tx_gas_price B256
);

wrap_hostio!(
    /// Gets the top-level sender of the transaction. The semantics are equivalent to that of the
    /// EVM's [`ORIGIN`] opcode.
    ///
    /// [`ORIGIN`]: https://www.evm.codes/#32
    origin tx_origin Address
);
