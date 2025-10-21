// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{primitives::Bytes, sol};
    use eyre::Result;
    use stylus_tools::devnet::Node;

    sol! {
        #[sol(rpc)]
        interface IMultiCall {
            function multicall(address[] memory addresses, bytes[] memory data) external view returns (bytes[] memory);
            error ArraySizeNotMatch();
            error CallFailed(uint256);
        }
    }

    #[tokio::test]
    async fn errors() -> Result<()> {
        let devnode = Node::new().await?;
        let rpc = devnode.rpc();
        println!("Deploying contract to Nitro ({rpc})...");
        let (address, _) = stylus_tools::Deployer::new(rpc.to_owned()).deploy()?;
        println!("Deployed contract to {address}");
        let provider = devnode.create_provider().await?;
        let contract = IMultiCall::IMultiCallInstance::new(address, provider);

        let addresses = vec![];
        let data = vec![Bytes::new()];
        let err = contract
            .multicall(addresses, data)
            .call()
            .await
            .unwrap_err()
            .as_decoded_interface_error::<IMultiCall::IMultiCallErrors>()
            .unwrap();
        assert!(matches!(
            err,
            IMultiCall::IMultiCallErrors::ArraySizeNotMatch(..)
        ));

        Ok(())
    }
}
