// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{primitives::address, sol};
    use eyre::Result;
    use stylus_tools::utils::testing::init_test;

    sol! {
        #[sol(rpc)]
        interface IContract {
            function init() external;
            function owner() external view returns (address);
        }
    }

    const EXPECTED_ABI: &str = "\
interface IContract {
    function init() external;

    function owner() external view returns (address);
}";

    #[tokio::test]
    async fn constants() -> Result<()> {
        let (devnode, address) = init_test(EXPECTED_ABI).await?;
        let provider = devnode.create_provider().await?;

        // Instantiate contract
        let contract = IContract::IContractInstance::new(address, provider);

        contract.init().send().await?.watch().await?;
        let owner = contract.owner().call().await?;
        assert_eq!(
            owner,
            address!("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")
        );

        Ok(())
    }
}
