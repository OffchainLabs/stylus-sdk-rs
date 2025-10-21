// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{primitives::U256, sol};
    use eyre::Result;
    use stylus_tools::devnet::Node;

    sol! {
        #[sol(rpc)]
        interface ICounter  {
            function get() external view returns (uint256);
            function setCount(uint256 count) external;
            function inc() external;
            function dec() external;
        }
    }

    #[tokio::test]
    async fn first_app() -> Result<()> {
        let devnode = Node::new().await?;
        let rpc = devnode.rpc();
        println!("Deploying contract to Nitro ({rpc})...");
        let (address, _) = stylus_tools::Deployer::new(rpc.to_owned()).deploy()?;
        println!("Deployed contract to {address}");
        let provider = devnode.create_provider().await?;
        let contract = ICounter::ICounterInstance::new(address, provider);

        let counter = contract.get().call().await?;
        assert_eq!(counter, U256::from(0));

        contract
            .setCount(U256::from(100))
            .send()
            .await?
            .watch()
            .await?;
        let counter = contract.get().call().await?;
        assert_eq!(counter, U256::from(100));

        contract.dec().send().await?.watch().await?;
        let counter = contract.get().call().await?;
        assert_eq!(counter, U256::from(99));

        contract.inc().send().await?.watch().await?;
        let counter = contract.get().call().await?;
        assert_eq!(counter, U256::from(100));

        Ok(())
    }
}
