// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{
        hex, network::TransactionBuilder, primitives::U256, providers::Provider,
        rpc::types::TransactionRequest, sol,
    };
    use eyre::Result;
    use stylus_tools::devnet::{addresses::OWNER, Node};

    sol! {
        #[sol(rpc)]
        interface IPaymentTracker {
            function getBalance(address account) external view returns (uint256);
            function getStats() external view returns (uint256, uint256, uint256);
        }
    }

    #[tokio::test]
    async fn fallback_receive() -> Result<()> {
        let devnode = Node::new().await?;
        let rpc = devnode.rpc();
        println!("Deploying contract to Nitro ({rpc})...");
        let (address, _, _) = stylus_tools::DeployerBuilder::default()
            .rpc(rpc)
            .build()?
            .deploy()?;
        println!("Deployed contract to {address}");
        let provider = devnode.create_provider().await?;
        let contract = IPaymentTracker::IPaymentTrackerInstance::new(address, &provider);

        // Call receive
        let tx = TransactionRequest::default()
            .with_to(*contract.address())
            .with_value(U256::from(100));
        provider.send_transaction(tx).await?.watch().await?;

        // Call fallback
        let tx = TransactionRequest::default()
            .with_to(*contract.address())
            .with_value(U256::from(100))
            .with_input(hex!("0xdeadbeef"));
        provider.send_transaction(tx).await?.watch().await?;

        // Check balance
        let balance = contract.getBalance(OWNER).call().await?;
        assert_eq!(balance, U256::from(200));

        Ok(())
    }
}
