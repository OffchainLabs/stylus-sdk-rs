// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::sol;
    use eyre::Result;
    use stylus_tools::devnet::{addresses::OWNER, Node};

    sol! {

        #[sol(rpc)]
        interface INestedStructs  {
            function addUser(address _address, string calldata name) external;
            function addDogs(address user, Dog[] memory dogs) external;
            function getUser(address _address) external view returns (User);
            function getAllUsers() external view returns (User[] memory);
            error NotFound();
            error AlreadyExists();
            error InvalidParam();

            #[derive(Debug, PartialEq, Eq)]
            struct Dog {string name;string breed;}

            #[derive(Debug, PartialEq, Eq)]
            struct User {address _address;string name;Dog[] dogs;}
        }
    }

    const EXPECTED_ABI: &str = "\
interface INestedStructs {
    function addUser(address _address, string calldata name) external;

    function addDogs(address user, Dog[] memory dogs) external;

    function getUser(address _address) external view returns (User);

    function getAllUsers() external view returns (User[] memory);

    error NotFound();

    error AlreadyExists();

    error InvalidParam();

    struct Dog {string name;string breed;}

    struct User {address _address;string name;Dog[] dogs;}
}";
    const EXPECTED_CONSTRUCTOR: &str = "";

    #[tokio::test]
    async fn nested_structs() -> Result<()> {
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
        let contract = INestedStructs::INestedStructsInstance::new(address, provider);

        let dogs = vec![
            INestedStructs::Dog {
                name: "jojo".to_owned(),
                breed: "maltese".to_owned(),
            },
            INestedStructs::Dog {
                name: "zeze".to_owned(),
                breed: "chihuahua".to_owned(),
            },
        ];

        contract
            .addUser(OWNER, "foobar".to_owned())
            .send()
            .await?
            .watch()
            .await?;

        contract
            .addDogs(OWNER, dogs.clone())
            .send()
            .await?
            .watch()
            .await?;

        let users = contract.getAllUsers().call().await?;
        assert_eq!(
            users,
            vec![INestedStructs::User {
                _address: OWNER,
                name: "foobar".into(),
                dogs,
            }]
        );

        Ok(())
    }
}
