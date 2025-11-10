// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{
        primitives::{address, U256},
        sol,
    };
    use eyre::Result;
    use stylus_tools::utils::testing::init_test;

    sol! {
        #[sol(rpc)]
        interface IMapping {
            function getMyMap(address target) external view returns (bool);
            function setMyMap(address target, bool new_value) external;
            function removeMyMap(address target) external;
            function getMyNestedMap(uint256 index, address target) external view returns (bool);
            function setMyNestedMap(uint256 index, address target, bool new_value) external;
            function removeMyNestedMap(uint256 index, address target) external;
        }
    }

    const EXPECTED_ABI: &str = "\
interface IMapping {
    function getMyMap(address target) external view returns (bool);

    function setMyMap(address target, bool new_value) external;

    function removeMyMap(address target) external;

    function getMyNestedMap(uint256 index, address target) external view returns (bool);

    function setMyNestedMap(uint256 index, address target, bool new_value) external;

    function removeMyNestedMap(uint256 index, address target) external;
}";

    #[tokio::test]
    async fn mapping() -> Result<()> {
        let (devnode, address) = init_test(EXPECTED_ABI).await?;
        let provider = devnode.create_provider().await?;

        // Instantiate contract
        let contract = IMapping::IMappingInstance::new(address, provider);

        let key = address!("0xfafafafafafafafafafafafafafafafafafafafa");
        contract.setMyMap(key, true).send().await?.watch().await?;
        let value = contract.getMyMap(key).call().await?;
        assert!(value);
        contract.removeMyMap(key).send().await?.watch().await?;
        let value = contract.getMyMap(key).call().await?;
        assert!(!value);

        let index = U256::from(10);
        contract
            .setMyNestedMap(index, key, true)
            .send()
            .await?
            .watch()
            .await?;
        let value = contract.getMyNestedMap(index, key).call().await?;
        assert!(value);
        contract
            .removeMyNestedMap(index, key)
            .send()
            .await?
            .watch()
            .await?;
        let value = contract.getMyNestedMap(index, key).call().await?;
        assert!(!value);

        Ok(())
    }
}
