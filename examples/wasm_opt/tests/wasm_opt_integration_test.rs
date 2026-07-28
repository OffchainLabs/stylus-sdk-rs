// Copyright 2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{primitives::U256, sol};
    use eyre::Result;
    use stylus_tools::utils::testing::init_test;

    sol! {
        #[sol(rpc)]
        interface ICounter {
            function get() external view returns (uint256);
            function setCount(uint256 count) external;
            function inc() external;
            function dec() external;
        }
    }

    const EXPECTED_ABI: &str = "\
interface ICounter {
    function get() external view returns (uint256);

    function setCount(uint256 count) external;

    function inc() external;

    function dec() external;
}";

    /// Deploys and verifies a contract that opts into the pinned `wasm-opt` step, then exercises
    /// it. `init_test` runs check -> deploy -> activate -> verify; the verify step re-applies the
    /// same `wasm-opt` recipe and byte-matches it against the deployed bytes, so a passing verify
    /// proves the optimization is reproducible. The counter operations afterward confirm the
    /// optimization preserved the contract's behavior.
    #[tokio::test]
    async fn wasm_opt() -> Result<()> {
        let (devnode, address) = init_test(EXPECTED_ABI).await?;
        let provider = devnode.create_provider().await?;

        let contract = ICounter::ICounterInstance::new(address, provider);

        let counter = contract.get().call().await?;
        assert_eq!(counter, U256::from(0));

        contract
            .setCount(U256::from(100))
            .send()
            .await?
            .watch()
            .await?;
        assert_eq!(contract.get().call().await?, U256::from(100));

        contract.inc().send().await?.watch().await?;
        assert_eq!(contract.get().call().await?, U256::from(101));

        contract.dec().send().await?.watch().await?;
        assert_eq!(contract.get().call().await?, U256::from(100));

        Ok(())
    }
}
