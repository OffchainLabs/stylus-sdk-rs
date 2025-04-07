// Copyright 2022-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

/// Returns the minimum number of EVM words needed to store `bytes` bytes.
pub(crate) const fn evm_words(bytes: usize) -> usize {
    (bytes + 31) / 32
}

/// Pads a length to the next multiple of 32 bytes
pub(crate) const fn evm_padded_length(bytes: usize) -> usize {
    evm_words(bytes) * 32
}
