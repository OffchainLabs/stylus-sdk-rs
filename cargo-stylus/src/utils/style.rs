// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use anstyle::{AnsiColor, Effects, Style};

pub const BOLD: Style = Style::new().effects(Effects::BOLD);
pub const ERROR: Style = AnsiColor::Red.on_default().effects(Effects::BOLD);
