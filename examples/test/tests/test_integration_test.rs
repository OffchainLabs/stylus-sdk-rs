// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{primitives::U256, sol};
    use eyre::Result;
    use stylus_tools::devnet::Node;

    sol! {
        #[sol(rpc)]
        interface ICounter {
            function owner() external view returns (address);
            function number() external view returns (uint256);
            function lastUpdated() external view returns (uint256);
            function setNumber(uint256 new_number) external;
            function mulNumber(uint256 new_number) external;
            function addNumber(uint256 new_number) external;
            function increment() external;
            function decrement() external;
            function addFromMsgValue() external payable;
            function callExternalContract(address target, uint8[] memory data) external returns (uint8[] memory);
            function transferOwnership(address new_owner) external;
        }
    }

    const EXPECTED_ABI: &str = "\
interface ICounter {
    function callingConsoleDoesntPanicInTest() external view;

    function owner() external view returns (address);

    function number() external view returns (uint256);

    function lastUpdated() external view returns (uint256);

    function setNumber(uint256 new_number) external;

    function mulNumber(uint256 new_number) external;

    function addNumber(uint256 new_number) external;

    function increment() external;

    function decrement() external;

    function addFromMsgValue() external payable;

    function callExternalContract(address target, uint8[] memory data) external returns (uint8[] memory);

    function transferOwnership(address new_owner) external;
}";
    const EXPECTED_CONSTRUCTOR: &str = "";

    #[tokio::test]
    async fn test() -> Result<()> {
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
        let contract = ICounter::ICounterInstance::new(address, provider);

        let number = contract.number().call().await?;
        assert_eq!(U256::ZERO, number);

        contract.increment().send().await?.watch().await?;
        let number = contract.number().call().await?;
        assert_eq!(U256::from(1), number);

        let estimate = contract.addNumber(U256::from(3)).estimate_gas().await?;
        contract
            .addNumber(U256::from(3))
            .gas((11 * estimate) / 10)
            .send()
            .await?
            .watch()
            .await?;
        let number = contract.number().call().await?;
        assert_eq!(U256::from(4), number);

        let estimate = contract.mulNumber(U256::from(2)).estimate_gas().await?;
        contract
            .mulNumber(U256::from(2))
            .gas((11 * estimate) / 10)
            .send()
            .await?
            .watch()
            .await?;
        let number = contract.number().call().await?;
        assert_eq!(U256::from(8), number);

        let estimate = contract.setNumber(U256::from(100)).estimate_gas().await?;
        contract
            .setNumber(U256::from(100))
            .gas((11 * estimate) / 10)
            .send()
            .await?
            .watch()
            .await?;
        let number = contract.number().call().await?;
        assert_eq!(U256::from(100), number);

        Ok(())
    }
}
