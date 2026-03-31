// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{
        primitives::{Bytes, U256},
        sol,
    };
    use eyre::Result;
    use stylus_tools::utils::testing::init_test;

    sol! {
        #[sol(rpc)]
        interface ITuples {
            function numbers() external returns (uint256, uint256, uint256);
            function bytesAndNumber() external returns (bytes memory, uint256);
        }
    }

    const EXPECTED_ABI: &str = "\
interface ITuples {
    function numbers() external returns (uint256, uint256, uint256);

    function bytesAndNumber() external returns (bytes memory, uint256);
}";

    #[tokio::test]
    async fn tuples() -> Result<()> {
        let (devnode, address) = init_test(EXPECTED_ABI).await?;
        let provider = devnode.create_provider().await?;

        // Instantiate contract
        let contract = ITuples::ITuplesInstance::new(address, provider);

        let numbers_return = contract.numbers().call().await?;
        assert_eq!(numbers_return._0, U256::from(100));
        assert_eq!(numbers_return._1, U256::from(200));
        assert_eq!(numbers_return._2, U256::from(300));

        let bytes_and_number_return = contract.bytesAndNumber().call().await?;
        assert_eq!(bytes_and_number_return._0, Bytes::from([1, 2, 3]));
        assert_eq!(bytes_and_number_return._1, U256::from(42));

        Ok(())
    }
}
