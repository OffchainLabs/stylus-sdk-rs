// Copyright 2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

//! Common imports for Stylus contracts.
//!
//! Included are all the proc macros and common traits.
//!
//! ```
//! use stylus_sdk::prelude::*;
//! ```

pub use crate::storage::{Erase, SimpleStorageType, StorageType, TopLevelStorage};
pub use crate::stylus_proc::*;
pub use crate::types::AddressVM;
