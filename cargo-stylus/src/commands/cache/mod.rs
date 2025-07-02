// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use crate::error::CargoStylusResult;

mod bid;
mod status;
mod suggest_bid;

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Places a bid on a Stylus contract to cache it in the Arbitrum chain's wasm cache manager.
    #[command(visible_alias = "b")]
    Bid(bid::Args),
    /// Checks the status of a Stylus contract in the Arbitrum chain's wasm cache manager.
    #[command(visible_alias = "s")]
    Status(status::Args),
    /// Checks the status of a Stylus contract in the Arbitrum chain's wasm cache manager.
    #[command()]
    SuggestBid(suggest_bid::Args),
}

pub async fn exec(cmd: Command) -> CargoStylusResult {
    match cmd {
        Command::Bid(args) => bid::exec(args).await,
        Command::Status(args) => status::exec(args).await,
        Command::SuggestBid(args) => suggest_bid::exec(args).await,
    }
}
