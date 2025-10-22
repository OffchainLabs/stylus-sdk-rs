// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::sol;
    use eyre::Result;
    use stylus_tools::devnet::Node;

    sol! {
        #[sol(rpc)]
        interface IData {
            function userMain(uint8[] memory _input) external pure returns (uint8[] memory);
        }
    }

    #[tokio::test]
    async fn primitive_data_types() -> Result<()> {
        let devnode = Node::new().await?;
        let rpc = devnode.rpc();
        println!("Deploying contract to Nitro ({rpc})...");
        let (address, _, _) = stylus_tools::DeployerBuilder::default()
            .rpc(rpc)
            .build()?
            .deploy()?;
        println!("Deployed contract to {address}");
        let provider = devnode.create_provider().await?;
        let contract = IData::IDataInstance::new(address, provider);

        let input = "hello".to_owned();
        let result = contract.userMain(input.into()).call().await?;
        assert_eq!(result, Vec::<u8>::new());

        Ok(())
    }
}
