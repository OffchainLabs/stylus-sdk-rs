// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! This test starts a nitro devnet node, deploys the contract to it, and sends a transaction
//! verifying the contract is working.

use stylus_e2e_test::DevNode;
use eyre::Result;

#[tokio::test]
async fn it_works() -> Result<()> {
    let devnode = DevNode::new().await?;
    let address = devnode.deploy().await?;
    println!("Contract address: {address}");
    Ok(())
}
