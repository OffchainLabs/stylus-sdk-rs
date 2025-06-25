// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{
        primitives::{address, U256},
        providers::Provider,
        sol,
    };
    use eyre::Result;
    use stylus_tools::devnet::Node;

    sol! {
        #[sol(rpc)]
        interface ISendEther {
            function sendViaTransfer(address to) external payable;
            function sendViaCall(address to) external payable;
            function sendViaCallGasLimit(address to, uint64 gas_amount) external payable;
            function sendViaCallWithCallData(address to, bytes calldata data) external payable;
            function sendToStylusContract(address to) external payable;
        }
    }

    #[tokio::test]
    async fn sending_ether() -> Result<()> {
        let devnode = Node::new().await?;
        let rpc = devnode.rpc();
        println!("Deploying contract to Nitro ({rpc})...");
        let address = stylus_tools::Deployer::new(rpc.to_owned()).deploy()?;
        println!("Deployed contract to {address}");
        let provider = devnode.create_provider().await?;
        let contract = ISendEther::ISendEtherInstance::new(address, &provider);

        let address = address!("0xfafafafafafafafafafafafafafafafafafafafa");
        let value = U256::from(100);
        contract
            .sendViaTransfer(address)
            .value(value)
            .send()
            .await?
            .watch()
            .await?;
        let balance = provider.get_balance(address).await?;
        assert_eq!(balance, value);

        Ok(())
    }
}
