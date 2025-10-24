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

    const EXPECTED_ABI: &str = "\
interface ISendEther {
    function sendViaTransfer(address to) external payable;

    function sendViaCall(address to) external payable;

    function sendViaCallGasLimit(address to, uint64 gas_amount) external payable;

    function sendViaCallWithCallData(address to, bytes calldata data) external payable;

    function sendToStylusContract(address to) external payable;
}";
    const EXPECTED_CONSTRUCTOR: &str = "";

    #[tokio::test]
    async fn sending_ether() -> Result<()> {
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
