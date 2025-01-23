// Copyright 2024-2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

/// Trait for top-level storage types, usually implemented by proc macros.
/// Top-level types are special in that their lifetimes track the entirety
/// of all the EVM state-changes throughout a contract invocation.
///
/// To prevent storage aliasing during reentrancy, you must hold a reference
/// to such a type when making an EVM call. This may change in the future
/// for programs that prevent reentrancy.
///
/// # Safety
///
/// The type must be top-level to prevent storage aliasing.
pub unsafe trait TopLevelStorage {}
