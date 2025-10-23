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

    #[tokio::test]
    async fn nested_structs() -> Result<()> {
        let devnode = Node::new().await?;
        let rpc = devnode.rpc();
        println!("Deploying contract to Nitro ({rpc})...");
        let (address, _, _) = stylus_tools::Deployer::builder()
            .rpc(rpc)
            .build()
            .deploy()?;
        println!("Deployed contract to {address}");
        let provider = devnode.create_provider().await?;
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
