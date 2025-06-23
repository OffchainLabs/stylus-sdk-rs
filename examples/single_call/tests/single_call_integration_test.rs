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
        interface ISingleCall  {
            function execute(address target, bytes calldata data) external view returns (bytes memory);
        }

        // ArbOwner precompile used for tests
        #[sol(rpc)]
        interface ArbOwnerPublic {
            function getAllChainOwners() external view returns (address[] memory);
        }
    }

    #[tokio::test]
    async fn single_call() -> Result<()> {
        let devnode = Node::new().await?;
        let rpc = devnode.rpc();
        println!("Deploying contract to Nitro ({rpc})...");
        let address = stylus_tools::Deployer::new(rpc.to_owned()).deploy()?;
        println!("Deployed contract to {address}");
        let provider = devnode.create_provider().await?;
        let contract = ISingleCall::ISingleCallInstance::new(address, provider);

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
