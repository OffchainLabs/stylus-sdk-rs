// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{primitives::U256, providers::Provider, sol, sol_types::SolCall};
    use eyre::Result;
    use stylus_tools::devnet::Node;

    sol! {
        #[sol(rpc)]
        interface IExampleContract {
            function lowLevelDelegateCall(bytes memory calldata, address target) external returns (uint8[] memory);
            function rawDelegateCall(uint8[] memory calldata, address target) external returns (uint8[] memory);
            error DelegateCallFailed();
        }

        // solc v0.8.29; solc Storage.sol --via-ir --optimize --bin
        #[sol(rpc, bytecode="608080604052346013576094908160188239f35b5f80fdfe60808060405260043610156011575f80fd5b5f3560e01c80636057361d14604857638381f58a14602d575f80fd5b346044575f3660031901126044576020905f548152f35b5f80fd5b3460445760203660031901126044576004355f5500fea26469706673582212205a8c00b582dff04b92a9d9bddba71af8dc085cbace4e12705bdcbfc1e57fe73e64736f6c634300081d0033")]
        contract Storage {
            uint256 public number;

            function store(uint256 num) public {
                number = num;
            }
        }
    }
    //TODO: calldata is the generated param type in lowLevelDelegateCall
    const EXPECTED_ABI: &str = "\
interface IExampleContract {
    function lowLevelDelegateCall(bytes calldata calldata, address target) external returns (uint8[] memory);

    function rawDelegateCall(uint8[] memory calldata, address target) external returns (uint8[] memory);

    error DelegateCallFailed();
}";
    const EXPECTED_CONSTRUCTOR: &str = "";

    #[tokio::test]
    async fn delegate_call() -> Result<()> {
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
        let contract = IExampleContract::IExampleContractInstance::new(address, &provider);

        // deploy storage contract
        let storage = Storage::deploy(&provider).await?;

        let store_calldata = Storage::storeCall {
            num: U256::from(123),
        }
        .abi_encode();
        contract
            .lowLevelDelegateCall(store_calldata.into(), storage.address().to_owned())
            .send()
            .await?
            .watch()
            .await?;

        let stored_number = provider
            .get_storage_at(contract.address().to_owned(), U256::ZERO)
            .await?;
        assert_eq!(stored_number, U256::from(123));

        Ok(())
    }
}
