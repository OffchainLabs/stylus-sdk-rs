// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;

use alloc::string::String;
use alloy_primitives::FixedBytes;
use alloy_sol_types::{
    sol,
    sol_data::{Address as SOLAddress, FixedBytes as SolFixedBytes, *},
    SolType,
};
/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{
    abi::Bytes,
    alloy_primitives::{address, Address, U256},
    crypto::keccak,
    prelude::*,
};

type ECRECOVERType = (
    SolFixedBytes<32>,
    Uint<8>,
    SolFixedBytes<32>,
    SolFixedBytes<32>,
);

sol! {
    error EcrecoverCallError();
    error InvalidSignatureLength();
}

// Define some persistent storage using the Solidity ABI.
// `VerifySignature` will be the entrypoint.
#[storage]
#[entrypoint]
pub struct VerifySignature;

#[derive(SolidityError)]
pub enum VerifySignatureError {
    EcrecoverCallError(EcrecoverCallError),
    InvalidSignatureLength(InvalidSignatureLength),
}

const ECRECOVER: Address = address!("0000000000000000000000000000000000000001");
const SIGNED_MESSAGE_HEAD: &str = "\x19Ethereum Signed Message:\n32";

/// Declare that `VerifySignature` is a contract with the following external methods.
#[public]
impl VerifySignature {
    /* 1. Unlock MetaMask account
    ethereum.enable()
    */

    /* 2. Get message hash to sign
    getMessageHash(
        0x14723A09ACff6D2A60DcdF7aA4AFf308FDDC160C,
        123,
        "coffee and donuts",
        1
    )

    hash = "0xcf36ac4f97dc10d91fc2cbb20d718e94a8cbfe0f82eaedc6a4aa38946fb797cd"
    */
    pub fn get_message_hash(
        &self,
        to: Address,
        amount: U256,
        message: String,
        nonce: U256,
    ) -> FixedBytes<32> {
        let message_data: &[&[u8]] = &[
            to.as_ref(),
            &amount.to_be_bytes_vec(),
            message.as_bytes(),
            &nonce.to_be_bytes_vec(),
        ];
        keccak(message_data.concat())
    }

    /* 3. Sign message hash
    # using browser
    account = "copy paste account of signer here"
    ethereum.request({ method: "personal_sign", params: [account, hash]}).then(console.log)

    # using web3
    web3.personal.sign(hash, web3.eth.defaultAccount, console.log)

    Signature will be different for different accounts
    0x993dab3dd91f5c6dc28e17439be475478f5635c92a56e17e82349d3fb2f166196f466c0b4e0c146f285204f0dcb13e5ae67bc33f4b888ec32dfe0a063e8f3f781b
    */
    pub fn get_eth_signed_message_hash(&self, message_hash: FixedBytes<32>) -> FixedBytes<32> {
        let message_to_be_decoded =
            [SIGNED_MESSAGE_HEAD.as_bytes(), message_hash.as_ref()].concat();
        keccak(message_to_be_decoded)
    }

    /* 4. Verify signature
    signer = 0xB273216C05A8c0D4F0a4Dd0d7Bae1D2EfFE636dd
    to = 0x14723A09ACff6D2A60DcdF7aA4AFf308FDDC160C
    amount = 123
    message = "coffee and donuts"
    nonce = 1
    signature =
        0x993dab3dd91f5c6dc28e17439be475478f5635c92a56e17e82349d3fb2f166196f466c0b4e0c146f285204f0dcb13e5ae67bc33f4b888ec32dfe0a063e8f3f781b
    */
    pub fn verify(
        &self,
        signer: Address,
        to: Address,
        amount: U256,
        message: String,
        nonce: U256,
        signature: Bytes,
    ) -> Result<bool, VerifySignatureError> {
        let message_hash = self.get_message_hash(to, amount, message, nonce);
        let eth_signed_message_hash = self.get_eth_signed_message_hash(message_hash);
        match self.recover_signer(eth_signed_message_hash, signature) {
            Ok(recovered_signer) => Ok(recovered_signer == signer),
            Err(err) => Err(err),
        }
    }

    pub fn recover_signer(
        &self,
        eth_signed_message_hash: FixedBytes<32>,
        signature: Bytes,
    ) -> Result<Address, VerifySignatureError> {
        let (r, s, v) = self.split_signature(signature);
        self.ecrecover_call(eth_signed_message_hash, v, r, s)
    }

    /// Invoke the ECRECOVER precompile.
    pub fn ecrecover_call(
        &self,
        hash: FixedBytes<32>,
        v: u8,
        r: FixedBytes<32>,
        s: FixedBytes<32>,
    ) -> Result<Address, VerifySignatureError> {
        let data = (hash, v, r, s);
        let encoded_data = ECRECOVERType::abi_encode(&data);
        match static_call(self.vm(), Call::new(), ECRECOVER, &encoded_data) {
            Ok(result) => Ok(SOLAddress::abi_decode(&result).unwrap()),
            Err(_) => Err(VerifySignatureError::EcrecoverCallError(
                EcrecoverCallError {},
            )),
        }
    }

    pub fn split_signature(&self, signature: Bytes) -> (FixedBytes<32>, FixedBytes<32>, u8) {
        let r = FixedBytes::from_slice(&signature[0..32]);
        let s = FixedBytes::from_slice(&signature[32..64]);
        let v = signature[64];
        (r, s, v)
    }
}
