// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{
        primitives::{fixed_bytes, I256, U256},
        sol,
    };
    use eyre::Result;
    use stylus_tools::devnet::addresses::OWNER;
    use stylus_tools::utils::testing::init_test;

    sol! {
        #[sol(rpc)]
        interface IData  {
            function getBool() external view returns (bool);
            function getAddress() external view returns (address);
            function getUint() external view returns (uint256);
            function getSigned() external view returns (int256);
            function getFixedBytes() external view returns (bytes4);
            function getBytes() external view returns (uint8[] memory);
            function getByteFromBytes(uint256 index) external view returns (uint8);
            function getString() external view returns (string memory);
            function getVec(uint256 index) external view returns (uint256);
            function setBool(bool value) external;
            function setAddress(address value) external;
            function setUint(uint256 value) external;
            function setSigned(int256 value) external;
            function setFixedBytes(bytes4 value) external;
            function setBytes(uint8[] memory value) external;
            function pushByteToBytes(uint8 value) external;
            function setString(string calldata value) external;
            function pushVec(uint256 value) external;
        }
    }

    const EXPECTED_ABI: &str = "\
interface IData {
    function getBool() external view returns (bool);

    function getAddress() external view returns (address);

    function getUint() external view returns (uint256);

    function getSigned() external view returns (int256);

    function getFixedBytes() external view returns (bytes4);

    function getBytes() external view returns (uint8[] memory);

    function getByteFromBytes(uint256 index) external view returns (uint8);

    function getString() external view returns (string memory);

    function getVec(uint256 index) external view returns (uint256);

    function setBool(bool value) external;

    function setAddress(address value) external;

    function setUint(uint256 value) external;

    function setSigned(int256 value) external;

    function setFixedBytes(bytes4 value) external;

    function setBytes(uint8[] memory value) external;

    function pushByteToBytes(uint8 value) external;

    function setString(string calldata value) external;

    function pushVec(uint256 value) external;
}";

    #[tokio::test]
    async fn storage_data_types() -> Result<()> {
        let (devnode, address) = init_test(EXPECTED_ABI).await?;
        let provider = devnode.create_provider().await?;

        // Instantiate contract
        let contract = IData::IDataInstance::new(address, provider);

        contract.setBool(true).send().await?.watch().await?;
        let value = contract.getBool().call().await?;
        assert!(value);

        contract.setAddress(OWNER).send().await?.watch().await?;
        let value = contract.getAddress().call().await?;
        assert_eq!(value, OWNER);

        contract
            .setUint(U256::from(123))
            .send()
            .await?
            .watch()
            .await?;
        let value = contract.getUint().call().await?;
        assert_eq!(value, U256::from(123));

        contract
            .setSigned(I256::unchecked_from(-123))
            .send()
            .await?
            .watch()
            .await?;
        let value = contract.getSigned().call().await?;
        assert_eq!(value, I256::unchecked_from(-123));

        contract
            .setFixedBytes(fixed_bytes!("0xdeadbeef"))
            .send()
            .await?
            .watch()
            .await?;
        let value = contract.getFixedBytes().call().await?;
        assert_eq!(value, fixed_bytes!("0xdeadbeef"));

        contract
            .setBytes(vec![10, 20, 30])
            .send()
            .await?
            .watch()
            .await?;
        let value = contract.getBytes().call().await?;
        assert_eq!(value, vec![10, 20, 30]);

        contract.pushByteToBytes(40).send().await?.watch().await?;
        let value = contract.getByteFromBytes(U256::from(3)).call().await?;
        assert_eq!(value, 40);

        contract
            .setString("hello".to_owned())
            .send()
            .await?
            .watch()
            .await?;
        let value = contract.getString().call().await?;
        assert_eq!(value, "hello".to_owned());

        contract
            .pushVec(U256::from(123))
            .send()
            .await?
            .watch()
            .await?;
        let value = contract.getVec(U256::from(0)).call().await?;
        assert_eq!(value, U256::from(123));

        Ok(())
    }
}
