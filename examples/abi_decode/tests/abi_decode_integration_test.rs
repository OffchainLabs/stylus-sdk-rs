// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{
        primitives::{address, U256},
        sol,
    };
    use eyre::Result;
    use stylus_tools::devnet::Node;

    sol! {
        #[sol(rpc)]
        interface IDecoder {
            function encodeAndDecode(address _address, uint256 amount) external view returns (bool);
            error DecodedFailed();
        }
    }

    #[tokio::test]
    async fn abi_decode() -> Result<()> {
        let devnode = Node::new().await?;
        let rpc = devnode.rpc();
        println!("Deploying contract to Nitro ({rpc})...");
        let (address, _) = stylus_tools::Deployer::new(rpc.to_owned()).deploy()?;
        println!("Deployed contract to {address}");
        let provider = devnode.create_provider().await?;
        let contract = IDecoder::IDecoderInstance::new(address, provider);

        let address = address!("0xfafafafafafafafafafafafafafafafafafafafa");
        let amount = U256::from(1234);
        let result = contract.encodeAndDecode(address, amount).call().await?;
        assert!(result);

        Ok(())
    }
}
