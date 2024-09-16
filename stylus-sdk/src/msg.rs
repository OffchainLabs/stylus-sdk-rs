// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! VM affordances for inspecting the current call.
//!
//! See also [`block`](crate::block), [`contract`](crate::contract), [`evm`](crate::evm),
//! [`msg`](crate::msg), and [`tx`](crate::tx).
//!
//! ```no_run
//! use stylus_sdk::msg;
//!
//! let call_value = msg::value();
//! ```

use crate::hostio::{self, wrap_hostio};
use alloy_primitives::{Address, B256, U256};

wrap_hostio!(
    /// Whether the current call is reentrant.
    reentrant REENTRANT msg_reentrant bool
);

wrap_hostio!(
    /// Gets the address of the account that called the program. For normal L2-to-L2 transactions
    /// the semantics are equivalent to that of the EVM's [`CALLER`] opcode, including in cases
    /// arising from [`DELEGATE_CALL`].
    ///
    /// For L1-to-L2 retryable ticket transactions, the top-level sender's address will be aliased.
    /// See [`Retryable Ticket Address Aliasing`] for more information on how this works.
    ///
    /// [`CALLER`]: https://www.evm.codes/#33
    /// [`DELEGATE_CALL`]: https://www.evm.codes/#f4
    /// [`Retryable Ticket Address Aliasing`]: https://developer.arbitrum.io/arbos/l1-to-l2-messaging#address-aliasing
    sender SENDER msg_sender Address
);

wrap_hostio!(
    /// Get the ETH value in wei sent to the program.
    value VALUE msg_value U256
);
