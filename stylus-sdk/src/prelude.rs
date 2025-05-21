// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Common imports for Stylus contracts.
//!
//! Included are all the proc macros and common traits.
//!
//! ```
//! use stylus_sdk::prelude::*;
//! ```

pub use crate::call::*;
pub use crate::storage::{Erase, SimpleStorageType, StorageType};
pub use crate::stylus_core::*;
pub use crate::stylus_proc::*;
