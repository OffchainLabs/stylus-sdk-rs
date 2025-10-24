// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{
        primitives::{Bytes, U256},
        sol,
    };
    use eyre::Result;
    use stylus_tools::devnet::Node;

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
    const EXPECTED_CONSTRUCTOR: &str = "";

    #[tokio::test]
    async fn tuples() -> Result<()> {
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
