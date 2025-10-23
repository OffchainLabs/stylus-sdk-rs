// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{
        primitives::{utils::parse_ether, U256},
        providers::Provider,
        sol,
    };
    use eyre::Result;
    use stylus_tools::devnet::{addresses::OWNER, Node};

    sol! {
        #[sol(rpc)]
        interface IContract {
            function setNumber(uint256 number) external;
            function number() external view returns (uint256);
            function owner() external view returns (address);
            error Unauthorized();
        }
    }

    const EXPECTED_ABI: &str = "\
interface IContract {
    function setNumber(uint256 number) external;

    function number() external view returns (uint256);

    function owner() external view returns (address);

    error Unauthorized();
}";
    const EXPECTED_CONSTRUCTOR: &str = "constructor(uint256 initial_number) payable";

    #[tokio::test]
    async fn constructor() -> Result<()> {
        let exporter = stylus_tools::Exporter::builder().build();
        assert_eq!(exporter.export_abi()?, EXPECTED_ABI);
        assert_eq!(exporter.export_constructor()?, EXPECTED_CONSTRUCTOR);

        let devnode = Node::new().await?;
        let rpc = devnode.rpc();

        println!("Checking contract on Nitro ({rpc})...");
        stylus_tools::Checker::builder().rpc(rpc).build().check()?;
        println!("Checked contract");

        let deployer = stylus_tools::Deployer::builder()
            .rpc(rpc.to_owned())
            .constructor_args(vec!["0xbeef".to_owned()])
            .constructor_value("12.34".to_owned())
            .build();
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

        let verify = stylus_tools::Verifier::builder()
            .rpc(rpc)
            .dir("../callee".to_owned())
            .deployment_tx_hash(tx_hash.to_string())
            .build()
            .verify();
        assert!(verify.is_err(), "Should fail verifying wrong contract");
        println!("Verified contract with tx hash {tx_hash}");

        let provider = devnode.create_provider().await?;

        // Check balance sent in constructor
        let balance = provider.get_balance(address).await?;
        assert_eq!(balance, parse_ether("12.34")?);
        println!("Got balance: {balance}");

        let contract = IContract::IContractInstance::new(address, provider);

        // Check values set by constructor
        let owner = contract.owner().call().await?;
        assert_eq!(owner, OWNER);
        let number = contract.number().call().await?;
        assert_eq!(number, U256::from(0xbeef));

        // Change number and check
        let new_number = U256::from(123);
        contract.setNumber(new_number).send().await?.watch().await?;
        let number = contract.number().call().await?;
        assert_eq!(number, new_number);

        Ok(())
    }
}
