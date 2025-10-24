// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{primitives::U256, sol};
    use eyre::Result;
    use stylus_tools::devnet::{addresses::OWNER, Node};

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

    const EXPECTED_ABI: &str = "\
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
    const EXPECTED_CONSTRUCTOR: &str = "";

    #[tokio::test]
    async fn arrays() -> Result<()> {
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
