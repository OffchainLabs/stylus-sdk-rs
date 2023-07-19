// Copyright 2023, Offchain Labs, Inc.
// For license information, see https://github.com/OffchainLabs/nitro/blob/master/LICENSE

use crate::hostio::{self, wrap_hostio};
use alloy_primitives::{Address, B256, U64};

wrap_hostio!(
    /// Gets the basefee of the current block.
    basefee block_basefee B256
);

wrap_hostio!(
    /// Gets the unique chain identifier of the Arbitrum chain.
    chainid chainid B256
);

wrap_hostio!(
    /// Gets the coinbase of the current block, which on Arbitrum chains is the L1 batch poster's
    /// address.
    coinbase block_coinbase Address
);

wrap_hostio!(
    /// Gets the gas limit of the current block.
    gas_limit block_gas_limit U64
);

wrap_hostio!(
    /// Gets a bounded estimate of the L1 block number at which the Sequencer sequenced the
    /// transaction. See [`Block Numbers and Time`] for more information on how this value is
    /// determined.
    ///
    /// [`Block Numbers and Time`]: https://developer.arbitrum.io/time
    number block_number B256
);

wrap_hostio!(
    /// Gets a bounded estimate of the Unix timestamp at which the Sequencer sequenced the
    /// transaction. See [`Block Numbers and Time`] for more information on how this value is
    /// determined.
    ///
    /// [`Block Numbers and Time`]: https://developer.arbitrum.io/time
    timestamp block_timestamp U64
);
