// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::sol;
    use eyre::Result;
    use stylus_tools::utils::testing::init_test;

    sol! {
        #[sol(rpc)]
        interface IHello {
            function userMain(uint8[] memory _input) external pure returns (uint8[] memory);
        }
    }

    const EXPECTED_ABI: &str = "\
interface IHello {
    function userMain(uint8[] memory _input) external pure returns (uint8[] memory);
}";

    #[tokio::test]
    async fn hello_world() -> Result<()> {
        let (devnode, address) = init_test(EXPECTED_ABI).await?;
        let provider = devnode.create_provider().await?;

        // Instantiate contract
        let contract = IHello::IHelloInstance::new(address, provider);

        let input = "hello".to_owned();
        let result = contract.userMain(input.into()).call().await?;
        assert_eq!(result, Vec::<u8>::new());

        Ok(())
    }
}
