// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{
        primitives::{address, U256},
        sol,
    };
    use eyre::Result;
    use stylus_tools::devnet::{Node, DEVNET_PRIVATE_KEY};

    sol! {
        #[sol(rpc)]
        interface IMapping {
            function getMyMap(address target) external view returns (bool);
            function setMyMap(address target, bool new_value) external;
            function removeMyMap(address target) external;
            function getMyNestedMap(uint256 index, address target) external view returns (bool);
            function setMyNestedMap(uint256 index, address target, bool new_value) external;
            function removeMyNestedMap(uint256 index, address target) external;
        }
    }

    #[tokio::test]
    async fn mapping() -> Result<()> {
        let devnode = Node::new().await?;
        let rpc = devnode.rpc();
        println!("Deploying contract to Nitro ({rpc})...");
        let address = stylus_tools::deploy(rpc, DEVNET_PRIVATE_KEY)?;
        println!("Deployed contract to {address}");
        let provider = devnode.create_provider().await?;
        let contract = IMapping::IMappingInstance::new(address, provider);

        let key = address!("0xfafafafafafafafafafafafafafafafafafafafa");
        contract.setMyMap(key, true).send().await?.watch().await?;
        let value = contract.getMyMap(key).call().await?;
        assert!(value);
        contract.removeMyMap(key).send().await?.watch().await?;
        let value = contract.getMyMap(key).call().await?;
        assert!(!value);

        let index = U256::from(10);
        contract
            .setMyNestedMap(index, key, true)
            .send()
            .await?
            .watch()
            .await?;
        let value = contract.getMyNestedMap(index, key).call().await?;
        assert!(value);
        contract
            .removeMyNestedMap(index, key)
            .send()
            .await?
            .watch()
            .await?;
        let value = contract.getMyNestedMap(index, key).call().await?;
        assert!(!value);

        Ok(())
    }
}
