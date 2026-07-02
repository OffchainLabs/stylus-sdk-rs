// Copyright 2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{providers::Provider, sol};
    use eyre::Result;
    use stylus_tools::{core::code::prefixes::ROOT_NO_DICT, devnet::Node};

    sol! {
        #[sol(rpc)]
        interface ILargeContract {
            function blobHash() external view returns (bytes32);
            function blobLen() external view returns (uint64);
        }
    }

    const EXPECTED_ABI: &str = "\
interface ILargeContract {
    function blobHash() external view returns (bytes32);

    function blobLen() external view returns (uint64);
}";

    /// Size of the blob embedded in the contract (see `src/lib.rs`).
    const BLOB_LEN: u64 = 48 * 1024;

    /// Deploys the intentionally-large contract (which is split into multiple fragments) and
    /// verifies it end-to-end. This exercises `cargo stylus verify`'s fragmented-deployment path.
    #[tokio::test]
    async fn large_contract() -> Result<()> {
        let exporter = stylus_tools::Exporter::builder().build();
        assert_eq!(exporter.export_abi()?, EXPECTED_ABI);

        let devnode = Node::new().await?;
        let rpc = devnode.rpc();

        println!("Checking contract on Nitro ({rpc})...");
        stylus_tools::Checker::builder().rpc(rpc).build().check()?;
        println!("Checked contract");

        println!("Deploying contract to Nitro ({rpc})...");
        let (address, tx_hash, _gas_used) = stylus_tools::Deployer::builder()
            .rpc(rpc)
            .build()
            .deploy()?;
        println!("Deployed contract to {address} (tx {tx_hash})");

        let provider = devnode.create_provider().await?;

        // Sanity check: the deployment must actually have fragmented, otherwise this test would
        // silently exercise the single-contract path instead of the fragment path. A fragmented
        // deployment installs a *root* contract at the address, identified by the ROOT prefix.
        let code = provider.get_code_at(address).await?;
        assert!(
            code.starts_with(ROOT_NO_DICT),
            "expected a fragmented (root) deployment; got code prefix {:?}",
            &code[..code.len().min(4)],
        );

        // Verify the fragmented deployment against the local source build.
        let verify = stylus_tools::Verifier::builder()
            .rpc(rpc)
            .deployment_tx_hash(tx_hash.to_string())
            .build()
            .verify();
        assert!(
            verify.is_ok(),
            "failed to verify fragmented contract: {verify:?}"
        );
        println!("Verified fragmented contract with tx hash {tx_hash}");

        // The deployed contract still functions: the embedded blob survived deployment.
        let contract = ILargeContract::ILargeContractInstance::new(address, provider);
        let len = contract.blobLen().call().await?;
        assert_eq!(len, BLOB_LEN);

        Ok(())
    }
}
