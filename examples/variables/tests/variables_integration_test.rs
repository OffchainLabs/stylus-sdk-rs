// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::sol;
    use eyre::Result;
    use stylus_tools::utils::testing::init_test;

    sol! {
        #[sol(rpc)]
        interface IContract {
            function init() external;
            function doSomething() external view;
        }
    }

    const EXPECTED_ABI: &str = "\
interface IContract {
    function init() external;

    function doSomething() external view;
}";

    #[tokio::test]
    async fn variables() -> Result<()> {
        let (devnode, address) = init_test(EXPECTED_ABI).await?;
        let provider = devnode.create_provider().await?;

        // Instantiate contract
        let contract = IContract::IContractInstance::new(address, provider);

        contract.init().send().await?.watch().await?;
        contract.doSomething().call().await?;

        Ok(())
    }
}
