// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{
        primitives::{address, fixed_bytes, U256},
        sol,
    };
    use eyre::Result;
    use stylus_tools::devnet::Node;

    sol! {
        #[sol(rpc)]
        interface IHasher {
            function encodeAndHash(address target, uint256 value, string calldata func, bytes calldata data, uint256 timestamp) external view returns (bytes32);
            function encodeAndDecode(address _address, uint256 amount) external view returns (bool);
            function packedEncodeAndHash1(address target, uint256 value, string calldata func, bytes calldata data, uint256 timestamp) external view returns (bytes32);
            function packedEncodeAndHash2(address target, uint256 value, string calldata func, bytes calldata data, uint256 timestamp) external view returns (bytes32);
            function encodeWithSignature(string calldata func, address _address, uint256 amount) external view returns (uint8[] memory);
            function encodeWithSignatureAndHash(string calldata func, address _address, uint256 amount) external view returns (bytes32);
            error DecodedFailed();
        }
    }

    #[tokio::test]
    async fn encoding_and_hashing() -> Result<()> {
        let devnode = Node::new().await?;
        let rpc = devnode.rpc();
        println!("Deploying contract to Nitro ({rpc})...");
        let (address, _, _) = stylus_tools::DeployerBuilder::default()
            .rpc(rpc)
            .build()?
            .deploy()?;
        println!("Deployed contract to {address}");
        let provider = devnode.create_provider().await?;
        let contract = IHasher::IHasherInstance::new(address, provider);

        let address = address!("0xfafafafafafafafafafafafafafafafafafafafa");
        let value = U256::from(0xdead);
        let func = "foo()".to_owned();
        let data = vec![];
        let timestamp = U256::from(0xbeef);

        let hash = contract
            .encodeAndHash(address, value, func.clone(), data.clone().into(), timestamp)
            .call()
            .await?;
        assert_eq!(
            hash,
            fixed_bytes!("d78779d27306eaa45371c59710ac58bc2c6585a62e6091d5c1b71354f7c25a22")
        );

        Ok(())
    }
}
