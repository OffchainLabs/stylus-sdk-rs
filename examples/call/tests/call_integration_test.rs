// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{
        primitives::{address, Address},
        sol,
        sol_types::SolCall,
    };
    use eyre::Result;
    use stylus_tools::devnet::{addresses::OWNER, Node};

    const ARB_OWNER_PUBLIC: Address = address!("0x000000000000000000000000000000000000006b");

    sol! {
        #[sol(rpc)]
        interface IExampleContract  {
            function execute(address target, bytes calldata data) external view returns (bytes memory);
            function simpleCall(address account, address user) external returns (string memory);
            function callWithGasValue(address account, address user) external payable returns (string memory);
            function callPure(address methods) external view;
            function callView(address methods) external view;
            function callWrite(address methods) external;
            function callPayable(address methods) external payable;
            function makeGenericCall(address account, address user) external returns (string memory);
            function executeCall(address _contract, uint8[] memory calldata) external returns (uint8[] memory);
            function executeStaticCall(address _contract, uint8[] memory calldata) external returns (uint8[] memory);
            function rawCallExample(address _contract, uint8[] memory calldata) external returns (uint8[] memory);
        }

        // ArbOwner precompile used for tests
        #[sol(rpc)]
        interface ArbOwnerPublic {
            function getAllChainOwners() external view returns (address[] memory);
        }
    }

    const EXPECTED_ABI: &str = "\
interface IExampleContract {
    function execute(address target, bytes calldata data) external view returns (bytes memory);

    function simpleCall(address account, address user) external returns (string memory);

    function callWithGasValue(address account, address user) external payable returns (string memory);

    function callPure(address methods) external view;

    function callView(address methods) external view;

    function callWrite(address methods) external;

    function callPayable(address methods) external payable;

    function makeGenericCall(address account, address user) external returns (string memory);

    function executeCall(address _contract, uint8[] memory calldata) external returns (uint8[] memory);

    function executeStaticCall(address _contract, uint8[] memory calldata) external returns (uint8[] memory);

    function rawCallExample(address _contract, uint8[] memory calldata) external returns (uint8[] memory);
}";
    const EXPECTED_CONSTRUCTOR: &str = "";

    #[tokio::test]
    async fn call() -> Result<()> {
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
        let contract = IExampleContract::IExampleContractInstance::new(address, provider);

        let calldata = ArbOwnerPublic::getAllChainOwnersCall {}.abi_encode();
        let owners_raw = contract
            .execute(ARB_OWNER_PUBLIC, calldata.into())
            .call()
            .await?;
        let owners =
            ArbOwnerPublic::getAllChainOwnersCall::abi_decode_returns_validate(&owners_raw)?;

        assert_eq!(owners, vec![Address::ZERO, OWNER]);

        Ok(())
    }
}
