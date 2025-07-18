// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::sol;
    use eyre::Result;
    use stylus_tools::devnet::Node;

    sol! {
        #[sol(rpc)]
        interface ITupleWithVariableSizesElements {
            function onlyString() external view returns (string memory);

            function onlyVec() external view returns (uint8[] memory);

            function u256AndU256() external view returns (uint256, uint256);

            function u256AndString() external view returns (uint256, string memory);

            function u256AndVec() external view returns (uint256, uint8[] memory);
        }
    }

    #[tokio::test]
    async fn tuple_with_variable_sizes_elements() -> Result<()> {
        let devnode = Node::new().await?;
        let rpc = devnode.rpc();
        let provider = devnode.create_provider().await?;

        println!("Deploying contract to Nitro ({rpc})...");
        let contract_address = stylus_tools::Deployer::new(rpc.to_owned()).deploy()?;
        println!("Deployed contract to {contract_address}");

        let contract = ITupleWithVariableSizesElements::new(contract_address, provider);

        // SUCCEEDS
        let ret_only_string = contract.onlyString().call().await?;
        println!("onlyString returned: {ret_only_string}");

        // SUCCEEDS
        let ret_only_vec = contract.onlyVec().call().await?;
        println!("onlyVec returned: {:?}", ret_only_vec);

        // SUCCEEDS
        let ret_u256_and_u256 = contract.u256AndU256().call().await?;
        println!(
            "u256AndU256 returned: ({}, {})",
            ret_u256_and_u256._0, ret_u256_and_u256._1
        );

        // FAILS: type check failed for "offset (usize)" with data: 0000000000000000000000000000000000000000002a00000000000000000000
        let ret_u256_and_string = contract.u256AndString().call().await?;
        println!(
            "u256AndString returned: ({}, {})",
            ret_u256_and_string._0, ret_u256_and_string._1
        );

        // FAILS: ABI decoding failed: buffer overrun while deserializing
        let ret_u256_and_vec = contract.u256AndVec().call().await?;
        println!(
            "u256AndVec returned: ({}, {:?})",
            ret_u256_and_vec._0, ret_u256_and_vec._1
        );

        Ok(())
    }
}
