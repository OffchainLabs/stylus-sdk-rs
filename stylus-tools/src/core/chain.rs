// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

/// Maximum code size per EIP-170
pub const DEFAULT_MAX_CODE_SIZE: u64 = 24_576;

#[derive(Debug)]
pub struct ChainConfig {
    pub max_code_size: u64,
}

impl Default for ChainConfig {
    fn default() -> Self {
        Self {
            max_code_size: DEFAULT_MAX_CODE_SIZE,
        }
    }
}
