use crate::devnet::Node;
use crate::{Activator, Checker, Deployer, Exporter, Verifier};
use alloy::primitives::{Address, TxHash};
use eyre::Result;

fn test_export(constructor: &str, abi: &str) -> Result<()> {
    let exporter = Exporter::builder().build();
    assert_eq!(exporter.export_abi()?, abi);
    assert_eq!(exporter.export_constructor()?, constructor);
    Ok(())
}

fn test_check(rpc: &str) -> Result<()> {
    println!("Checking contract on Nitro ({rpc})...");
    Checker::builder().rpc(rpc).build().check()?;
    println!("Checked contract");
    Ok(())
}

fn test_deploy(rpc: &str) -> Result<(Address, TxHash)> {
    let deployer = Deployer::builder().rpc(rpc).build();
    println!("Estimating gas...");
    let gas_estimate = deployer.estimate_gas()?;
    println!("Estimated deployment gas: {gas_estimate} ETH");

    println!("Deploying contract to Nitro ({rpc})...");
    let (address, tx_hash, gas_used) = deployer.deploy()?;
    println!("Deployed contract to {address}");

    // Approximate equality is usually expected, but given the test conditions, the gas estimate equals the gas used
    assert_eq!(gas_used, gas_estimate);
    Ok((address, tx_hash))
}

fn test_activate(rpc: &str, address: Address) -> Result<()> {
    println!("Activating contract at {address} on Nitro ({rpc})...");
    Activator::builder()
        .rpc(rpc)
        .contract_address(address.to_string())
        .build()
        .activate()?;
    println!("Activated contract at {address}");
    Ok(())
}

fn test_verify(rpc: &str, tx_hash: TxHash) -> Result<()> {
    let verify = Verifier::builder()
        .rpc(rpc)
        .deployment_tx_hash(tx_hash.to_string())
        .build()
        .verify();
    assert!(verify.is_ok(), "Failed to verify contract");
    Ok(())
}

pub async fn init_test(expected_abi: &str) -> Result<(Node, Address)> {
    test_export("", expected_abi)?;

    let devnode = Node::new().await?;
    let rpc = devnode.rpc();

    test_check(rpc)?;

    let (address, tx_hash) = test_deploy(rpc)?;

    test_activate(rpc, address)?;

    test_verify(rpc, tx_hash)?;

    Ok((devnode, address))
}
