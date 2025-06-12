// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::sol;
    use eyre::Result;
    use stylus_tools::devnet::{Node, DEVNET_PRIVATE_KEY};

    sol! {
        #[sol(rpc)]
        interface IContract {
            function init() external;
            function doSomething() external view;
        }
    }

    #[tokio::test]
    async fn variables() -> Result<()> {
        let devnode = Node::new().await?;
        let rpc = devnode.rpc();
        println!("Deploying contract to Nitro ({rpc})...");
        let address = stylus_tools::deploy(rpc, DEVNET_PRIVATE_KEY)?;
        println!("Deployed contract to {address}");
        let provider = devnode.create_provider().await?;
        let contract = IContract::IContractInstance::new(address, provider);

        contract.init().send().await?.watch().await?;
        contract.doSomething().call().await?;

        Ok(())
    }
}
