// Copyright 2023, Offchain Labs, Inc.
// For license information, see https://github.com/OffchainLabs/nitro/blob/master/LICENSE

use crate::hostio::{self, wrap_hostio};
use alloy_primitives::{Address, B256, U64};

wrap_hostio!(
    /// Gets the gas price in wei per gas, which on Arbitrum chains equals the basefee.
    gas_price tx_gas_price B256
);

wrap_hostio!(
    /// Gets the price of ink in evm gas basis points. See [`Ink and Gas`] for more information on
    /// Stylus's compute-pricing model.
    ///
    /// [`Ink and Gas`]: https://developer.arbitrum.io/TODO
    ink_price tx_ink_price U64
);

wrap_hostio!(
    /// Gets the top-level sender of the transaction. The semantics are equivalent to that of the
    /// EVM's [`ORIGIN`] opcode.
    ///
    /// [`ORIGIN`]: https://www.evm.codes/#32
    origin tx_origin Address
);
