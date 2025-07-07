// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

// #[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{
        primitives::{address, Address, FixedBytes, U256},
        sol,
        sol_types::SolCall,
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
        }
    }

    #[tokio::test]
    async fn caller_callee() -> Result<()> {
        let devnode = Node::new().await?;
        let rpc = devnode.rpc();
        let provider = devnode.create_provider().await?;

        println!("Deploying callee contract to Nitro ({rpc})...");
        let callee_address = stylus_tools::Deployer::new(rpc.to_owned())
            .with_contract_dir("../callee".into())
            .deploy()?;
        println!("Deployed callee contract to {callee_address}");

        println!("Deploying caller contract to Nitro ({rpc})...");
        let caller_address = stylus_tools::Deployer::new(rpc.to_owned()).deploy()?;
        println!("Deployed caller contract to {caller_address}");

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
        assert_eq!(ret_multiple_inputs_multiple_outputs._1, true);
        assert_eq!(ret_multiple_inputs_multiple_outputs._2, callee_address);
        assert_eq!(
            ret_multiple_inputs_multiple_outputs._3,
            FixedBytes::from([0x01; 32]),
        );

        let ret_mutable = caller.mutable(callee_address).call().await?;
        assert!(ret_mutable);

        let call_ret = caller.fails(callee_address).call().await;
        assert!(call_ret.is_err(), "Expected call to fail, but it succeeded");

        // let hey = String::from("hey");
        // // let res_multiple_inputs_multiple_outputs = caller.multipleInputsStrMultipleOutputs(callee_address, U256::from(10), U256::from(10), hey).call().await;
        // // let res_multiple_inputs_multiple_outputs = caller.multipleInputsStrMultipleOutputs(callee_address, U256::from(10), U256::from(10)).call().await;
        // let res_multiple_inputs_multiple_outputs = caller.multipleInputsStrMultipleOutputs(callee_address, U256::from(10), U256::from(10)).call().await;
        // match res_multiple_inputs_multiple_outputs {
        //     Ok(res) => println!("multipleInputsMultipleOutputs result ({}, {})", res._0, res._1),
        //     Err(e) => {
        //         println!("Error calling multipleInputsMultipleOutputs: {}", e);
        //
        //         let stdout = devnode.container().stdout_to_vec().await?;
        //         let s = std::str::from_utf8(&stdout).expect("Invalid UTF-8");
        //         println!("Devnode stdout: {}", s);
        //
        //         let stderr = devnode.container().stderr_to_vec().await?;
        //         let s2 = std::str::from_utf8(&stderr).expect("Invalid UTF-8");
        //         println!("Devnode stderr: {}", s2);
        //     }
        // }

        Ok(())
    }
}
