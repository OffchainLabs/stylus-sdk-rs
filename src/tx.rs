// Copyright 2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use crate::hostio::{self, wrap_hostio};
use alloy_primitives::{Address, B256};

wrap_hostio! {
    /// Gets the price of ink in evm gas basis points. See [`Ink and Gas`] for more information on
    /// Stylus's compute-pricing model.
    ///
    /// [`Ink and Gas`]: https://developer.arbitrum.io/TODO
    ink_price CACHED_INK_PRICE tx_ink_price u64
}

/// Converts evm gas to ink. See [`Ink and Gas`] for more information on
/// Stylus's compute-pricing model.
///
/// [`Ink and Gas`]: https://developer.arbitrum.io/TODO
#[allow(clippy::inconsistent_digit_grouping)]
pub fn gas_to_ink(gas: u64) -> u64 {
    gas.saturating_mul(100_00) / ink_price()
}

/// Converts ink to evm gas. See [`Ink and Gas`] for more information on
/// Stylus's compute-pricing model.
///
/// [`Ink and Gas`]: https://developer.arbitrum.io/TODO
#[allow(clippy::inconsistent_digit_grouping)]
pub fn ink_to_gas(ink: u64) -> u64 {
    ink.saturating_mul(ink_price()) / 100_00
}

wrap_hostio!(
    /// Gets the gas price in wei per gas, which on Arbitrum chains equals the basefee.
    gas_price CACHED_GAS_PRICE tx_gas_price B256
);

wrap_hostio!(
    /// Gets the top-level sender of the transaction. The semantics are equivalent to that of the
    /// EVM's [`ORIGIN`] opcode.
    ///
    /// [`ORIGIN`]: https://www.evm.codes/#32
    origin CACHED_ORIGIN tx_origin Address
);
