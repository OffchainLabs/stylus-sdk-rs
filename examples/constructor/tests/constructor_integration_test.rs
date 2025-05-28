// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{
        providers::Provider,
        primitives::{utils::parse_ether, U256},
        sol,
    };
    use eyre::Result;
    use stylus_tools::devnet::{addresses::OWNER, Node, DEVNET_PRIVATE_KEY};

    sol! {
        #[sol(rpc)]
        interface IContract {
            function setNumber(uint256 number) external;
            function number() external view returns (uint256);
            function owner() external view returns (address);
            error Unauthorized();
        }
    }

    #[tokio::test]
    async fn constructor() -> Result<()> {
        let devnode = Node::new().await?;
        let rpc = devnode.rpc();
        println!("Deploying contract to Nitro ({rpc})...");
        let args = &["0xbeef"];
        let address = stylus_tools::deploy_with_constructor(rpc, DEVNET_PRIVATE_KEY, "12.34", args)?;
        println!("Deployed contract to {address}");
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
