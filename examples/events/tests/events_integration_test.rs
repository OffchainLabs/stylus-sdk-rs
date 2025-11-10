// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{primitives::Address, sol};
    use eyre::Result;
    use stylus_tools::utils::testing::init_test;

    sol! {
        #[sol(rpc)]
        interface IEvents {
            function userMain(uint8[] memory _input) external view returns (uint8[] memory);
        }

        event Log(address indexed sender, string message);
        event AnotherLog();
    }

    const EXPECTED_ABI: &str = "\
interface IEvents {
    function userMain(uint8[] memory _input) external view returns (uint8[] memory);
}";

    #[tokio::test]
    async fn events() -> Result<()> {
        let (devnode, address) = init_test(EXPECTED_ABI).await?;
        let provider = devnode.create_provider().await?;

        // Instantiate contract
        let contract = IEvents::IEventsInstance::new(address, provider);

        let tx = contract
            .userMain(vec![])
            .send()
            .await?
            .get_receipt()
            .await?;

        let log = tx.decoded_log::<Log>().unwrap();
        assert_eq!(log.data.sender, Address::from([0x11; 20]));
        assert_eq!(log.data.message, "Hello world!");

        let another_log = tx.decoded_log::<AnotherLog>();
        assert!(another_log.is_some());

        Ok(())
    }
}
