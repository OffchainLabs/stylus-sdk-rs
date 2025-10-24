// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{primitives::Address, sol};
    use eyre::Result;
    use stylus_tools::devnet::Node;

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
    const EXPECTED_CONSTRUCTOR: &str = "";

    #[tokio::test]
    async fn events() -> Result<()> {
        let exporter = stylus_tools::Exporter::builder().build();
        assert_eq!(exporter.export_abi()?, EXPECTED_ABI);
        assert_eq!(exporter.export_constructor()?, EXPECTED_CONSTRUCTOR);

        let devnode = Node::new().await?;
        let rpc = devnode.rpc();

        println!("Checking contract on Nitro ({rpc})...");
        stylus_tools::Checker::builder().rpc(rpc).build().check()?;
        println!("Checked contract");

        let deployer = stylus_tools::Deployer::builder().rpc(rpc).build();
        println!("Estimating gas...");
        let gas_estimate = deployer.estimate_gas()?;
        println!("Estimated deployment gas: {gas_estimate} ETH");

        println!("Deploying contract to Nitro ({rpc})...");
        let (address, tx_hash, gas_used) = deployer.deploy()?;
        println!("Deployed contract to {address}");

        // Approximate equality is usually expected, but given the test conditions, the gas estimate equals the gas used
        assert_eq!(gas_used, gas_estimate);

        println!("Activating contract at {address} on Nitro ({rpc})...");
        stylus_tools::Activator::builder()
            .rpc(rpc)
            .contract_address(address.to_string())
            .build()
            .activate()?;
        println!("Activated contract at {address}");

        let verify = stylus_tools::Verifier::builder()
            .rpc(rpc)
            .deployment_tx_hash(tx_hash.to_string())
            .build()
            .verify();
        assert!(verify.is_ok(), "Failed to verify contract");
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
