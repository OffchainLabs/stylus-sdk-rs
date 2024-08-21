// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! VM affordances for inspecting the current tx.
//!
//! See also [`block`](crate::block), [`contract`](crate::contract), [`crypto`](crate::crypto),
//! [`evm`](crate::evm), and [`msg`](crate::msg).
//!
//! ```no_run
//! use stylus_sdk::tx;
//!
//! let gas_price = tx::gas_price();
//! ```

use crate::hostio::{self, wrap_hostio};
use alloy_primitives::{Address, B256, U256};

wrap_hostio! {
    /// Gets the price of ink in evm gas basis points. See [`Ink and Gas`] for more information on
    /// Stylus's compute-pricing model.
    ///
    /// [`Ink and Gas`]: https://docs.arbitrum.io/stylus/concepts/stylus-gas
    ink_price INK_PRICE tx_ink_price u32
}

/// Converts evm gas to ink. See [`Ink and Gas`] for more information on
/// Stylus's compute-pricing model.
///
/// [`Ink and Gas`]: https://docs.arbitrum.io/stylus/concepts/stylus-gas
pub fn gas_to_ink(gas: u64) -> u64 {
    gas.saturating_mul(ink_price().into())
}

/// Converts ink to evm gas. See [`Ink and Gas`] for more information on
/// Stylus's compute-pricing model.
///
/// [`Ink and Gas`]: https://docs.arbitrum.io/stylus/concepts/stylus-gas
pub fn ink_to_gas(ink: u64) -> u64 {
    ink / ink_price() as u64
}

wrap_hostio!(
    /// Gets the gas price in wei per gas, which on Arbitrum chains equals the basefee.
    gas_price GAS_PRICE tx_gas_price U256
);

wrap_hostio!(
    /// Gets the top-level sender of the transaction. The semantics are equivalent to that of the
    /// EVM's [`ORIGIN`] opcode.
    ///
    /// [`ORIGIN`]: https://www.evm.codes/#32
    origin ORIGIN tx_origin Address
);
