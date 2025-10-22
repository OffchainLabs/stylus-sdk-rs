// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{
        primitives::{address, bytes, fixed_bytes, U256},
        sol,
    };
    use eyre::Result;
    use stylus_tools::devnet::Node;

    sol! {
        #[sol(rpc)]
        interface IVerifySignature {
            function getMessageHash(address to, uint256 amount, string calldata message, uint256 nonce) external view returns (bytes32);
            function getEthSignedMessageHash(bytes32 message_hash) external view returns (bytes32);
            function verify(address signer, address to, uint256 amount, string calldata message, uint256 nonce, bytes calldata signature) external view returns (bool);
            function recoverSigner(bytes32 eth_signed_message_hash, bytes calldata signature) external view returns (address);
            function ecrecoverCall(bytes32 hash, uint8 v, bytes32 r, bytes32 s) external view returns (address);
            function splitSignature(bytes calldata signature) external view returns (bytes32, bytes32, uint8);
            error EcrecoverCallError();
            error InvalidSignatureLength();
        }
    }

    #[tokio::test]
    async fn verify_signature() -> Result<()> {
        let devnode = Node::new().await?;
        let rpc = devnode.rpc();
        println!("Deploying contract to Nitro ({rpc})...");
        let (address, _, _) = stylus_tools::DeployerBuilder::default()
            .rpc(rpc)
            .build()?
            .deploy()?;
        println!("Deployed contract to {address}");
        let provider = devnode.create_provider().await?;
        let contract = IVerifySignature::IVerifySignatureInstance::new(address, provider);

        let hash = contract
            .getMessageHash(
                address!("0x14723A09ACff6D2A60DcdF7aA4AFf308FDDC160C"),
                U256::from(123),
                "coffee and donuts".to_owned(),
                U256::from(1),
            )
            .call()
            .await?;
        assert_eq!(
            hash,
            fixed_bytes!("0xcf36ac4f97dc10d91fc2cbb20d718e94a8cbfe0f82eaedc6a4aa38946fb797cd")
        );

        let ok = contract
            .verify(
                address!("0xB273216C05A8c0D4F0a4Dd0d7Bae1D2EfFE636dd"),
                address!("0x14723A09ACff6D2A60DcdF7aA4AFf308FDDC160C"),
                U256::from(123),
                "coffee and donuts".into(),
                U256::from(1),
                bytes!("0x993dab3dd91f5c6dc28e17439be475478f5635c92a56e17e82349d3fb2f166196f466c0b4e0c146f285204f0dcb13e5ae67bc33f4b888ec32dfe0a063e8f3f781b")
            ).call()
            .await?;
        assert!(ok);

        Ok(())
    }
}
