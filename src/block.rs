// Copyright 2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use crate::hostio::{self, wrap_hostio};
use alloy_primitives::{Address, B256};

wrap_hostio!(
    /// Gets the basefee of the current block.
    basefee BASEFEE block_basefee B256
);

wrap_hostio!(
    /// Gets the unique chain identifier of the Arbitrum chain.
    chainid CHAINID chainid B256
);

wrap_hostio!(
    /// Gets the coinbase of the current block, which on Arbitrum chains is the L1 batch poster's
    /// address.
    coinbase COINBASE block_coinbase Address
);

wrap_hostio!(
    /// Gets the gas limit of the current block.
    gas_limit GAS_LIMIT block_gas_limit u64
);

wrap_hostio!(
    /// Gets a bounded estimate of the L1 block number at which the Sequencer sequenced the
    /// transaction. See [`Block Numbers and Time`] for more information on how this value is
    /// determined.
    ///
    /// [`Block Numbers and Time`]: https://developer.arbitrum.io/time
    number NUMBER block_number B256
);

wrap_hostio!(
    /// Gets a bounded estimate of the Unix timestamp at which the Sequencer sequenced the
    /// transaction. See [`Block Numbers and Time`] for more information on how this value is
    /// determined.
    ///
    /// [`Block Numbers and Time`]: https://developer.arbitrum.io/time
    timestamp TIMESTAMP block_timestamp u64
);
