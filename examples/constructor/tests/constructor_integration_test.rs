// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{
        primitives::{utils::parse_ether, U256},
        providers::Provider,
        sol,
    };
    use alloy_primitives::TxHash;
    use eyre::Result;
    use stylus_tools::devnet::addresses::OWNER;
    use stylus_tools::utils::testing::ExampleContractTester;
    use stylus_tools::{Deployer, Verifier};

    sol! {
        #[sol(rpc)]
        interface IContract {
            function setNumber(uint256 number) external;
            function number() external view returns (uint256);
            function owner() external view returns (address);
            error Unauthorized();
        }
    }

    struct ConstructorIntegrationTester {}

    impl ExampleContractTester for ConstructorIntegrationTester {
        const EXPECTED_ABI: &str = "\
interface IContract {
    function setNumber(uint256 number) external;

    function number() external view returns (uint256);

    function owner() external view returns (address);

    error Unauthorized();
}";
        const EXPECTED_CONSTRUCTOR: &'static str = "constructor(uint256 initial_number) payable";

        fn deployer(rpc: &str) -> Deployer {
            Deployer::builder()
                .rpc(rpc.to_owned())
                .constructor_args(vec!["0xbeef".to_owned()])
                .constructor_value("12.34".to_owned())
                .build()
        }

        fn test_verify(rpc: &str, tx_hash: TxHash) -> Result<()> {
            let verify = Verifier::builder()
                .rpc(rpc)
                .deployment_tx_hash(tx_hash.to_string())
                .build()
                .verify();
            assert!(verify.is_ok(), "Failed to verify contract");
            let verify = Verifier::builder()
                .rpc(rpc)
                .dir("../callee".to_owned())
                .deployment_tx_hash(tx_hash.to_string())
                .build()
                .verify();
            assert!(verify.is_err(), "Should fail verifying wrong contract");
            println!("Verified contract with tx hash {tx_hash}");
            Ok(())
        }
    }

    #[tokio::test]
    async fn constructor() -> Result<()> {
        let (devnode, address) = ConstructorIntegrationTester::init().await?;
        let provider = devnode.create_provider().await?;

        // Check balance sent in constructor
        let balance = provider.get_balance(address).await?;
        assert_eq!(balance, parse_ether("12.34")?);
        println!("Got balance: {balance}");

        let contract = IContract::IContractInstance::new(address, provider);

        // Check values set by constructor
        let owner = contract.owner().call().await?;
        assert_eq!(owner, OWNER);
        let number = contract.number().call().await?;
        assert_eq!(number, U256::from(0xbeef));

        // Change number and check
        let new_number = U256::from(123);
        contract.setNumber(new_number).send().await?.watch().await?;
        let number = contract.number().call().await?;
        assert_eq!(number, new_number);

        Ok(())
    }
}
