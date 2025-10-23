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

    #[tokio::test]
    async fn delegate_call() -> Result<()> {
        let devnode = Node::new().await?;
        let rpc = devnode.rpc();
        println!("Deploying contract to Nitro ({rpc})...");
        let (address, _, _) = stylus_tools::Deployer::builder()
            .rpc(rpc)
            .build()
            .deploy()?;
        println!("Deployed contract to {address}");
        let provider = devnode.create_provider().await?;
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
