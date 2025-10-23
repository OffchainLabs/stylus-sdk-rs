// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{
        primitives::{FixedBytes, U256},
        sol,
    };
    use eyre::Result;
    use stylus_tools::devnet::Node;

    sol! {
        #[sol(rpc)]
        interface ICaller  {
            function noInputNoOutput(address callee_addr) external view;

            function oneInputOneOutput(address callee_addr, uint256 input) external view returns (uint256);

            function multipleInputsMultipleOutputs(address callee_addr, uint256 input1, address input2) external view returns (uint256, bool, address, bytes32);

            function mutable(address callee_addr) external returns (bool);

            function fails(address callee_addr) external view;

            function outputsResultOk(address callee_addr) external view returns (uint256, uint256);

            function outputsResultErr(address callee_addr) external view returns (uint256);

            function outputsArbresultOk(address callee_addr) external view returns (uint8[] memory);

            function outputsArbresultErr(address callee_addr) external view returns (uint8[] memory);
        }
    }

    #[tokio::test]
    async fn caller_callee() -> Result<()> {
        let devnode = Node::new().await?;
        let rpc = devnode.rpc();
        let provider = devnode.create_provider().await?;

        println!("Deploying callee contract to Nitro ({rpc})...");
        let (callee_address, callee_deployment_tx_hash, _) = stylus_tools::Deployer::builder()
            .rpc(rpc)
            .dir("../callee".to_owned())
            .build()
            .deploy()?;
        println!("Deployed callee contract to {callee_address}");

        println!("Deploying caller contract to Nitro ({rpc})...");
        let (caller_address, caller_deployment_tx_hash, _) = stylus_tools::Deployer::builder()
            .rpc(rpc)
            .build()
            .deploy()?;
        println!("Deployed caller contract to {caller_address}");

        let verify = stylus_tools::Verifier::builder()
            .rpc(rpc)
            .deployment_tx_hash(caller_deployment_tx_hash.to_string())
            .build()
            .verify();
        assert!(verify.is_ok(), "Failed to verify caller contract");

        let verify = stylus_tools::Verifier::builder()
            .rpc(rpc)
            .deployment_tx_hash(callee_deployment_tx_hash.to_string())
            .build()
            .verify();
        assert!(verify.is_err(), "Provided wrong tx hash for verification");

        let verify = stylus_tools::Verifier::builder()
            .rpc(rpc)
            .dir("../callee".to_owned())
            .deployment_tx_hash(caller_deployment_tx_hash.to_string())
            .build()
            .verify();
        assert!(verify.is_err(), "Provided wrong contract for verification");

        let caller = ICaller::ICallerInstance::new(caller_address, provider);

        caller.noInputNoOutput(callee_address).call().await?;

        let ret_one_input_one_output = caller
            .oneInputOneOutput(callee_address, U256::from(10))
            .call()
            .await?;
        assert_eq!(ret_one_input_one_output, U256::from(11));

        let ret_multiple_inputs_multiple_outputs = caller
            .multipleInputsMultipleOutputs(callee_address, U256::from(10), callee_address)
            .call()
            .await?;
        assert_eq!(ret_multiple_inputs_multiple_outputs._0, U256::from(12));
        assert!(ret_multiple_inputs_multiple_outputs._1);
        assert_eq!(ret_multiple_inputs_multiple_outputs._2, callee_address);
        assert_eq!(
            ret_multiple_inputs_multiple_outputs._3,
            FixedBytes::from([0x01; 32]),
        );

        let ret_mutable = caller.mutable(callee_address).call().await?;
        assert!(ret_mutable);

        let ret_fails = caller.fails(callee_address).call().await;
        assert!(
            ret_fails.is_err(),
            "Expected call to fail, but it succeeded"
        );

        let ret_outputs_result_ok = caller.outputsResultOk(callee_address).call().await?;
        assert_eq!(ret_outputs_result_ok._0, U256::from(1234));
        assert_eq!(ret_outputs_result_ok._1, U256::from(5678));

        let ret_outputs_result_err = caller.outputsResultErr(callee_address).call().await;
        match ret_outputs_result_err {
            Err(e) => {
                assert!(e
                    .to_string()
                    .contains("execution reverted, data: \"0x010203\""));
            }
            Ok(_) => {
                panic!("Expected call to fail with specific error, but it succeeded");
            }
        }

        let ret_outputs_arbresult_ok = caller.outputsArbresultOk(callee_address).call().await?;
        assert_eq!(ret_outputs_arbresult_ok, Vec::<u8>::from([33, 34, 35]));

        let ret_outputs_arbresult_err = caller.outputsArbresultErr(callee_address).call().await;
        match ret_outputs_arbresult_err {
            Err(e) => {
                assert!(e
                    .to_string()
                    .contains("execution reverted, data: \"0x010203\""));
            }
            Ok(_) => {
                panic!("Expected call to fail with specific error, but it succeeded");
            }
        }

        Ok(())
    }
}
