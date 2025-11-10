// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{
        primitives::{address, U256},
        sol,
    };
    use eyre::Result;
    use stylus_tools::utils::testing::init_test;

    sol! {
        #[sol(rpc)]
        interface IDecoder {
            function encodeAndDecode(address _address, uint256 amount) external view returns (bool);
            error DecodedFailed();
        }
    }

    const EXPECTED_ABI: &str = "\
interface IDecoder {
    function encodeAndDecode(address _address, uint256 amount) external view returns (bool);

    error DecodedFailed();
}";

    #[tokio::test]
    async fn abi_decode() -> Result<()> {
        let (devnode, address) = init_test(EXPECTED_ABI).await?;
        let provider = devnode.create_provider().await?;

        // Instantiate contract
        let contract = IDecoder::IDecoderInstance::new(address, provider);

        let address = address!("0xfafafafafafafafafafafafafafafafafafafafa");
        let amount = U256::from(1234);
        let result = contract.encodeAndDecode(address, amount).call().await?;
        assert!(result);

        Ok(())
    }
}
