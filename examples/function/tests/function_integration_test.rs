// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{primitives::U256, sol};
    use eyre::Result;
    use stylus_tools::devnet::Node;

    sol! {
        #[sol(rpc)]
        interface IExampleContract {
            function setData(uint256 value) external;
            function getData() external view returns (uint256);
            function getOwner() external view returns (address);
        }
    }

    #[tokio::test]
    async fn function() -> Result<()> {
        let devnode = Node::new().await?;
        let rpc = devnode.rpc();
        println!("Deploying contract to Nitro ({rpc})...");
        let (address, _, _) = stylus_tools::Deployer::new(rpc.to_owned()).deploy()?;
        println!("Deployed contract to {address}");
        let provider = devnode.create_provider().await?;
        let contract = IExampleContract::IExampleContractInstance::new(address, provider);

        let data = U256::from(0xbeef);
        contract.setData(data).send().await?.watch().await?;
        let read = contract.getData().call().await?;
        assert_eq!(read, data);

        Ok(())
    }
}
