// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

// #[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{
        primitives::{address, Address, U256},
        sol,
        sol_types::SolCall,
    };
    use eyre::Result;
    use stylus_tools::devnet::Node;

    sol! {
        #[sol(rpc)]
        interface ICaller  {
            function noInputNoOutput(address callee_addr) external view;

            function noInputOneOutput(address callee_addr) external view returns (uint256);

            function noInputMultipleOutputs(address callee_addr) external view returns (uint256, uint256);

            function oneInputNoOutput(address callee_addr, uint256 input) external view;

            function oneInputOneOutput(address callee_addr, uint256 input) external view returns (uint256);

            function multipeInputsMultipleOutputs(address callee_addr, uint256 input1, string calldata input2) external view returns (uint256, string memory);

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
        let callee_address = stylus_tools::Deployer::new(rpc.to_owned()).with_contract_dir("../callee".into()).deploy()?;
        println!("Deployed callee contract to {callee_address}");

        println!("Deploying caller contract to Nitro ({rpc})...");
        let caller_address = stylus_tools::Deployer::new(rpc.to_owned()).deploy()?;
        println!("Deployed caller contract to {caller_address}");

        let caller = ICaller::ICallerInstance::new(caller_address, provider);

        caller.noInputNoOutput(callee_address).call().await?;

        let res_no_input_one_output = caller.noInputOneOutput(callee_address).call().await?;
        println!("noInputOneOutput result: {res_no_input_one_output}");

        let res_no_input_multiple_outputs = caller.noInputMultipleOutputs(callee_address).call().await?;
        println!("noInputMultipleOutputs result: ({}, {})", res_no_input_multiple_outputs._0, res_no_input_multiple_outputs._1);

        caller.oneInputNoOutput(callee_address, U256::from(10)).call().await?;

        let res_one_input_one_output = caller.oneInputOneOutput(callee_address, U256::from(10)).call().await?;
        println!("oneInputOneOutput result: {res_one_input_one_output}");

        let res_mutable = caller.mutable(callee_address).call().await?;
        println!("mutable result: {res_mutable}");

        // let address = stylus_tools::Deployer::new(rpc.to_owned()).deploy()?;
        // println!("Deployed contract to {address}");
        // let provider = devnode.create_provider().await?;
        // let contract = IExampleContract::IExampleContractInstance::new(address, provider);
        //
        // let calldata = ArbOwnerPublic::getAllChainOwnersCall {}.abi_encode();
        // let owners_raw = contract
        //     .execute(ARB_OWNER_PUBLIC, calldata.into())
        //     .call()
        //     .await?;
        // let owners =
        //     ArbOwnerPublic::getAllChainOwnersCall::abi_decode_returns_validate(&owners_raw)?;
        //
        // assert_eq!(owners, vec![Address::ZERO, OWNER]);

        Ok(())
    }
}
