// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{primitives::U256, sol};
    use eyre::Result;
    use stylus_tools::devnet::addresses::OWNER;
    use stylus_tools::utils::testing::ExampleContractTester;

    sol! {
        #[sol(rpc)]
        interface IArrays {
            function push(uint256 i) external;
            function getElement(uint256 index) external view returns (uint256);
            function getArrLength() external view returns (uint256);
            function remove(uint256 index) external;
            function getArr2Element(uint256 index) external view returns (uint256);
            function getArr2Length() external view returns (uint256);
            function setArr2Value(uint256 index, uint256 value) external;
            function pushArr3Info(uint256 value) external;
            function getArr3Length() external view returns (uint256);
            function getArr3Info(uint256 index) external view returns (address, uint256);
            function findArr3FirstExpectedValue(uint256 expected_value) external view returns (uint256);
        }
    }

    struct ArraysIntegrationTester {}

    impl ExampleContractTester for ArraysIntegrationTester {
        const EXPECTED_ABI: &'static str = "\
interface IArrays {
    function push(uint256 i) external;

    function getElement(uint256 index) external view returns (uint256);

    function getArrLength() external view returns (uint256);

    function remove(uint256 index) external;

    function getArr2Element(uint256 index) external view returns (uint256);

    function getArr2Length() external view returns (uint256);

    function setArr2Value(uint256 index, uint256 value) external;

    function pushArr3Info(uint256 value) external;

    function getArr3Length() external view returns (uint256);

    function getArr3Info(uint256 index) external view returns (address, uint256);

    function findArr3FirstExpectedValue(uint256 expected_value) external view returns (uint256);
}";
    }

    #[tokio::test]
    async fn arrays() -> Result<()> {
        let (devnode, address) = ArraysIntegrationTester::init().await?;
        let provider = devnode.create_provider().await?;

        // Instantiate contract
        let contract = IArrays::IArraysInstance::new(address, provider);

        contract
            .pushArr3Info(U256::from(10))
            .send()
            .await?
            .watch()
            .await?;
        contract
            .pushArr3Info(U256::from(20))
            .send()
            .await?
            .watch()
            .await?;
        contract
            .pushArr3Info(U256::from(30))
            .send()
            .await?
            .watch()
            .await?;
        let len = contract.getArr3Length().call().await?;
        assert_eq!(len, U256::from(3));

        let info = contract.getArr3Info(U256::from(2)).call().await?;
        assert_eq!(info._0, OWNER);
        assert_eq!(info._1, U256::from(30));

        let index = contract
            .findArr3FirstExpectedValue(U256::from(30))
            .call()
            .await?;
        assert_eq!(index, U256::from(2));

        Ok(())
    }
}
