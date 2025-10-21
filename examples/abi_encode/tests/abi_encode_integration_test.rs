// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{
        primitives::{address, hex, U256},
        sol,
    };
    use eyre::Result;
    use stylus_tools::devnet::Node;

    sol! {
        #[sol(rpc)]
        interface IEncoder {
            function encode(address target, uint256 value, string calldata func, bytes calldata data, uint256 timestamp) external view returns (uint8[] memory);
            function packedEncode(address target, uint256 value, string calldata func, bytes calldata data, uint256 timestamp) external view returns (uint8[] memory);
            function packedEncode2(address target, uint256 value, string calldata func, bytes calldata data, uint256 timestamp) external view returns (uint8[] memory);
            function encodeWithSignature(string calldata func, address _address, uint256 amount) external view returns (uint8[] memory);
        }
    }

    #[tokio::test]
    async fn abi_encode() -> Result<()> {
        let devnode = Node::new().await?;
        let rpc = devnode.rpc();
        println!("Deploying contract to Nitro ({rpc})...");
        let (address, _) = stylus_tools::Deployer::new(rpc.to_owned()).deploy()?;
        println!("Deployed contract to {address}");
        let provider = devnode.create_provider().await?;
        let contract = IEncoder::IEncoderInstance::new(address, provider);

        let address = address!("0xfafafafafafafafafafafafafafafafafafafafa");
        let value = U256::from(0xdead);
        let func = "foo()".to_owned();
        let data = vec![];
        let timestamp = U256::from(0xbeef);

        let encoded = contract
            .encode(address, value, func.clone(), data.clone().into(), timestamp)
            .call()
            .await?;
        assert_eq!(
            hex::encode(encoded),
            "000000000000000000000000fafafafafafafafafafafafafafafafafafafafa".to_owned()
                + "000000000000000000000000000000000000000000000000000000000000dead"
                + "00000000000000000000000000000000000000000000000000000000000000a0"
                + "00000000000000000000000000000000000000000000000000000000000000e0"
                + "000000000000000000000000000000000000000000000000000000000000beef"
                + "0000000000000000000000000000000000000000000000000000000000000005"
                + "666f6f2829000000000000000000000000000000000000000000000000000000"
                + "0000000000000000000000000000000000000000000000000000000000000000"
        );

        let encoded = contract
            .packedEncode(address, value, func.clone(), data.clone().into(), timestamp)
            .call()
            .await?;
        assert_eq!(
            hex::encode(encoded),
            "fafafafafafafafafafafafafafafafafafafafa".to_owned()
                + "000000000000000000000000000000000000000000000000000000000000dead"
                + "666f6f2829"
                + "000000000000000000000000000000000000000000000000000000000000beef"
        );

        let encoded = contract
            .packedEncode2(address, value, func.clone(), data.clone().into(), timestamp)
            .call()
            .await?;
        assert_eq!(
            hex::encode(encoded),
            "fafafafafafafafafafafafafafafafafafafafa".to_owned()
                + "000000000000000000000000000000000000000000000000000000000000dead"
                + "666f6f2829"
                + "000000000000000000000000000000000000000000000000000000000000beef"
        );

        Ok(())
    }
}
