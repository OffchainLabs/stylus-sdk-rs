// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{primitives::U256, sol};
    use eyre::Result;
    use stylus_tools::utils::testing::init_test;

    sol! {
        #[sol(rpc)]
        interface IExampleContract {
            function setData(uint256 value) external;
            function getData() external view returns (uint256);
            function getOwner() external view returns (address);
        }
    }

    const EXPECTED_ABI: &str = "\
interface IExampleContract {
    function setData(uint256 value) external;

    function getData() external view returns (uint256);

    function getOwner() external view returns (address);
}";

    #[tokio::test]
    async fn function() -> Result<()> {
        let (devnode, address) = init_test(EXPECTED_ABI).await?;
        let provider = devnode.create_provider().await?;

        // Instantiate contract
        let contract = IExampleContract::IExampleContractInstance::new(address, provider);

        let data = U256::from(0xbeef);
        contract.setData(data).send().await?.watch().await?;
        let read = contract.getData().call().await?;
        assert_eq!(read, data);

        Ok(())
    }
}
