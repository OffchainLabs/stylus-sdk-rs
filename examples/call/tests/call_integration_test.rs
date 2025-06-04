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
    use stylus_tools::devnet::{addresses::OWNER, Node, DEVNET_PRIVATE_KEY};

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

    #[tokio::test]
    async fn call() -> Result<()> {
        let devnode = Node::new().await?;
        let rpc = devnode.rpc();
        println!("Deploying contract to Nitro ({rpc})...");
        let address = stylus_tools::deploy(rpc, DEVNET_PRIVATE_KEY)?;
        println!("Deployed contract to {address}");
        let provider = devnode.create_provider().await?;
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
