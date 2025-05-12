// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::primitives::{Bytes, U256};

/// Length of prelude.
const INITCODE_LENGTH: usize = 42;

/// Length of included metadata.
const METADATA_LENGTH: usize = 1;

/// Total length of prelude (initcode + metadata), not including contract code.
const PRELUDE_LENGTH: usize = INITCODE_LENGTH + METADATA_LENGTH;

/// Calldata to send in deployment transaction.
#[derive(Debug)]
pub struct DeploymentCalldata(Vec<u8>);

impl DeploymentCalldata {
    /// Prepares an EVM bytecode prelude for contract creation.
    pub fn new(code: &[u8]) -> Self {
        let code_len: [u8; 32] = U256::from(code.len()).to_be_bytes();
        let mut deploy: Vec<u8> = Vec::with_capacity(code.len() + PRELUDE_LENGTH);
        deploy.push(0x7f); // PUSH32
        deploy.extend(code_len);
        deploy.push(0x80); // DUP1
        deploy.push(0x60); // PUSH1
        deploy.push(PRELUDE_LENGTH as u8); // prelude + version
        deploy.push(0x60); // PUSH1
        deploy.push(0x00);
        deploy.push(0x39); // CODECOPY
        deploy.push(0x60); // PUSH1
        deploy.push(0x00);
        deploy.push(0xf3); // RETURN
        deploy.push(0x00); // version
        deploy.extend(code);
        Self(deploy)
    }

    /// Extract and return EVM deployment prelude.
    pub fn prelude(&self) -> &[u8] {
        &self.0[..PRELUDE_LENGTH]
    }

    /// Extract and return compressed wasm code from calldata.
    pub fn compressed_wasm(&self) -> &[u8] {
        &self.0[PRELUDE_LENGTH..]
    }
}

impl From<DeploymentCalldata> for Bytes {
    fn from(calldata: DeploymentCalldata) -> Bytes {
        calldata.0.into()
    }
}
