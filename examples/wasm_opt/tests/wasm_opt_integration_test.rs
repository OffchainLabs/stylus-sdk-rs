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

    /// Directly exercises the pinned `wasm-opt` step. The deploy/verify test above is a symmetric
    /// self-comparison and would still pass if optimization were silently skipped on both sides;
    /// this fails unless `wasm-opt` actually ran and changed the bytes.
    #[test]
    fn wasm_opt_shrinks_module() -> Result<()> {
        use stylus_tools::core::optimize::{optimize, WasmOptConfig};

        // `$dead` is never exported or called, so `-Oz` dead-code-eliminates it and the output is
        // strictly smaller than the input.
        let wasm = wat::parse_str(
            r#"
            (module
              (func (export "main") (result i32)
                i32.const 42)
              (func $dead (result i32)
                (i32.add (i32.const 1)
                  (i32.add (i32.const 2)
                    (i32.add (i32.const 3)
                      (i32.add (i32.const 4) (i32.const 5)))))))
            "#,
        )?;

        let config = WasmOptConfig {
            version: "131".parse()?,
            flags: vec!["-Oz".to_string()],
        };
        let optimized = optimize(&wasm, &config)?;
        assert!(
            optimized.len() < wasm.len(),
            "wasm-opt did not shrink the module ({} -> {} bytes)",
            wasm.len(),
            optimized.len()
        );
        Ok(())
    }

    /// Loads this example's real `Stylus.toml` through resolution and pins the resolved recipe.
    /// The deploy/verify test is symmetric — it still passes if opt-in detection silently
    /// regresses to `None` on both sides — so this assertion is what makes such a regression
    /// fail the suite.
    #[test]
    fn wasm_opt_recipe_resolves_from_manifest() -> Result<()> {
        use cargo_metadata::MetadataCommand;
        use stylus_tools::core::{optimize::WasmOptConfig, project::contract::Contract};

        let metadata = MetadataCommand::new().no_deps().exec()?;
        let package = metadata
            .packages
            .iter()
            .find(|p| p.name.as_str() == "stylus-wasm-opt")
            .expect("example package present in workspace metadata");
        let contract = Contract::try_from(package)?;
        let config = WasmOptConfig::resolve_for_contract(&contract)?
            .expect("example's Stylus.toml opts into [wasm-opt]");
        assert_eq!(config.version.as_str(), "131");
        assert_eq!(config.flags, vec!["-Oz".to_string()]);
        Ok(())
    }
}
