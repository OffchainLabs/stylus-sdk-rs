// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{primitives::U256, sol};
    use eyre::Result;
    use stylus_tools::utils::testing::init_test;

    sol! {
        #[sol(rpc)]
        interface IContract {
            function setNumber(uint256 number) external;
            function number() external view returns (uint256);
        }
    }

    const EXPECTED_ABI: &str = "\
interface IContract {
    function setNumber(uint256 number) external;

    function number() external view returns (uint256);
}";

    #[tokio::test]
    async fn custom_storage_slots() -> Result<()> {
        let (devnode, address) = init_test(EXPECTED_ABI).await?;
        let provider = devnode.create_provider().await?;

        // Instantiate contract
        let contract = IContract::IContractInstance::new(address, provider);

        // Change number and check
        let new_number = U256::from(123);
        contract.setNumber(new_number).send().await?.watch().await?;
        let number = contract.number().call().await?;
        assert_eq!(number, new_number);

        Ok(())
    }
}
