// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::fmt;

use alloy::{
    consensus::Transaction,
    primitives::{Address, Bytes, TxHash},
    providers::Provider,
    sol_types::SolCall,
};

use crate::{
    core::{
        code::{
            contract::{ContractCode, RootContractError},
            fragments::{CodeFragment, CodeFragments},
            Code,
        },
        deployment::{
            deployer::{stylus_constructorCall, StylusDeployer::deployCall, ADDRESS},
            prelude::DeploymentCalldata,
        },
        project::contract::Contract,
        reflection,
        verification::VerificationError::{
            InvalidDeployerAddress, InvalidInitData, TransactionReceiptError, TxNotSuccessful,
        },
    },
    utils::cargo,
};

pub async fn verify(
    contract: &Contract,
    tx_hash: TxHash,
    skip_clean: bool,
    provider: &impl Provider,
) -> Result<VerificationStatus, VerificationError> {
    let tx = provider
        .get_transaction_by_hash(tx_hash)
        .await?
        .ok_or(VerificationError::NoCodeAtAddress)?;
    if !skip_clean {
        cargo::clean()?;
    }
    let deployment_success = provider
        .get_transaction_receipt(tx_hash)
        .await?
        .map(|receipt| receipt.status())
        .ok_or(TransactionReceiptError)?;
    if !deployment_success {
        return Err(TxNotSuccessful);
    }

    // Rebuild the contract locally and extract the code that the deployment transaction installed
    // on-chain. Both are independent of whether the contract fits in a single chunk or is split
    // into fragments.
    let status = contract.check(None, &Default::default(), provider).await?;
    let onchain = extract_deployment_calldata(&tx)?;

    match status.code() {
        Code::Contract(contract) => Ok(compare_calldata(
            &onchain,
            &DeploymentCalldata::new(contract.as_slice()),
        )),
        Code::Fragments(fragments) => verify_fragments(&onchain, fragments, provider).await,
    }
}

/// Extract the CREATE init code installed by a deployment transaction.
///
/// A plain deployment sends the init code directly (`tx.to()` is empty), while a deployment with a
/// constructor routes through the [`StylusDeployer`](deployCall), carrying the init code in the
/// `bytecode` field of the call. In both cases the returned calldata's [`compressed_wasm`] is the
/// code that ends up deployed on-chain (a single contract, or a fragment root).
///
/// [`compressed_wasm`]: DeploymentCalldata::compressed_wasm
fn extract_deployment_calldata(
    tx: &impl Transaction,
) -> Result<DeploymentCalldata, VerificationError> {
    match tx.to() {
        Some(deployer_address) => {
            reflection::constructor()?.ok_or(VerificationError::NoConstructor)?;
            let deploy_call = deployCall::abi_decode(tx.input())
                .map_err(|_| VerificationError::DeployCallDecode)?;
            let constructor_called = deploy_call
                .initData
                .starts_with(stylus_constructorCall::SELECTOR.as_slice());
            if !constructor_called {
                return Err(InvalidInitData);
            }
            if deployer_address != ADDRESS {
                return Err(InvalidDeployerAddress);
            }
            Ok(DeploymentCalldata(deploy_call.bytecode.to_vec()))
        }
        None => Ok(DeploymentCalldata(tx.input().to_vec())),
    }
}

/// Compare the on-chain deployment calldata against the locally built calldata for a single-chunk
/// contract.
fn compare_calldata(
    onchain: &DeploymentCalldata,
    local: &DeploymentCalldata,
) -> VerificationStatus {
    if onchain == local {
        return VerificationStatus::Success;
    }

    let prelude_mismatch = if onchain.prelude() == local.prelude() {
        None
    } else {
        Some(PreludeMismatch {
            tx: hex::encode(onchain.prelude()),
            build: hex::encode(local.prelude()),
        })
    };

    VerificationStatus::Failure(VerificationFailure::Contract {
        prelude_mismatch,
        tx_wasm_length: onchain.compressed_wasm().len(),
        build_wasm_length: local.compressed_wasm().len(),
    })
}

/// Verify a fragmented deployment.
///
/// The deployment transaction installs a *root* contract that records the uncompressed wasm size
/// and the addresses of the individual fragment contracts. We parse those out, fetch the code
/// deployed at each fragment address, and compare it against the locally built fragments.
async fn verify_fragments(
    onchain: &DeploymentCalldata,
    local: &CodeFragments,
    provider: &impl Provider,
) -> Result<VerificationStatus, VerificationError> {
    let (onchain_uncompressed_size, addresses) =
        ContractCode::parse_root_contract(onchain.compressed_wasm())?;

    let mut onchain_fragment_code = Vec::with_capacity(addresses.len());
    for address in &addresses {
        onchain_fragment_code.push(provider.get_code_at(*address).await?);
    }

    Ok(compare_fragments(
        local.uncompressed_wasm_size(),
        local.as_slice(),
        onchain_uncompressed_size,
        &addresses,
        &onchain_fragment_code,
    ))
}

/// Compare a locally built set of fragments against the code fetched from a fragmented deployment.
///
/// `onchain_addresses` and `onchain_fragment_code` are parallel: entry `i` is the address and the
/// deployed code of the `i`th fragment recorded in the root contract.
fn compare_fragments(
    local_uncompressed_size: u32,
    local_fragments: &[CodeFragment],
    onchain_uncompressed_size: u32,
    onchain_addresses: &[Address],
    onchain_fragment_code: &[Bytes],
) -> VerificationStatus {
    if onchain_uncompressed_size != local_uncompressed_size {
        return VerificationStatus::Failure(VerificationFailure::SizeMismatch {
            tx: onchain_uncompressed_size,
            build: local_uncompressed_size,
        });
    }

    if onchain_addresses.len() != local_fragments.len() {
        return VerificationStatus::Failure(VerificationFailure::FragmentCountMismatch {
            tx: onchain_addresses.len(),
            build: local_fragments.len(),
        });
    }

    for (index, ((onchain_code, address), local_fragment)) in onchain_fragment_code
        .iter()
        .zip(onchain_addresses)
        .zip(local_fragments)
        .enumerate()
    {
        let build = local_fragment.as_slice();
        if onchain_code.as_ref() != build {
            return VerificationStatus::Failure(VerificationFailure::FragmentMismatch {
                index,
                address: *address,
                tx_len: onchain_code.len(),
                build_len: build.len(),
                missing: onchain_code.is_empty(),
            });
        }
    }

    VerificationStatus::Success
}

#[derive(Debug)]
pub enum VerificationStatus {
    Success,
    Failure(VerificationFailure),
}

#[derive(Debug)]
pub enum VerificationFailure {
    /// A single-chunk contract's deployed code did not match the local build.
    Contract {
        prelude_mismatch: Option<PreludeMismatch>,
        tx_wasm_length: usize,
        build_wasm_length: usize,
    },
    /// The uncompressed wasm size recorded in the root contract did not match the local build.
    SizeMismatch { tx: u32, build: u32 },
    /// The number of fragments in the deployment did not match the local build.
    FragmentCountMismatch { tx: usize, build: usize },
    /// A fragment's deployed code did not match the local build (or was missing entirely).
    FragmentMismatch {
        index: usize,
        address: Address,
        tx_len: usize,
        build_len: usize,
        missing: bool,
    },
}

impl fmt::Display for VerificationFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Contract {
                prelude_mismatch,
                tx_wasm_length,
                build_wasm_length,
            } => {
                write!(f, "deployed code does not match local build")?;
                if let Some(mismatch) = prelude_mismatch {
                    write!(
                        f,
                        "; prelude mismatch (tx: {}, build: {})",
                        mismatch.tx, mismatch.build
                    )?;
                }
                write!(
                    f,
                    "; compressed wasm length (tx: {tx_wasm_length}, build: {build_wasm_length})"
                )
            }
            Self::SizeMismatch { tx, build } => {
                write!(
                    f,
                    "uncompressed wasm size mismatch (tx: {tx}, build: {build})"
                )
            }
            Self::FragmentCountMismatch { tx, build } => {
                write!(f, "fragment count mismatch (tx: {tx}, build: {build})")
            }
            Self::FragmentMismatch {
                index,
                address,
                tx_len,
                build_len,
                missing,
            } => {
                if *missing {
                    write!(
                        f,
                        "fragment {index} at {address} has no deployed code (expected {build_len} bytes)"
                    )
                } else {
                    write!(
                        f,
                        "fragment {index} at {address} does not match local build \
                         (tx: {tx_len} bytes, build: {build_len} bytes)"
                    )
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct PreludeMismatch {
    pub tx: String,
    pub build: String,
}

#[derive(Debug, thiserror::Error)]
pub enum VerificationError {
    #[error("RPC failed: {0}")]
    Rpc(#[from] alloy::transports::RpcError<alloy::transports::TransportErrorKind>),

    #[error("{0}")]
    Check(#[from] crate::core::check::CheckError),
    #[error("{0}")]
    Reflection(#[from] crate::core::reflection::ReflectionError),
    #[error("{0}")]
    Command(#[from] crate::error::CommandError),
    #[error("deployment transaction is not a well-formed fragmented contract: {0}")]
    RootContract(#[from] RootContractError),

    #[error("No code at address")]
    NoCodeAtAddress,
    #[error("Deployment transaction uses constructor but the local project doesn't have one")]
    NoConstructor,
    #[error("Failed to decode the deployer call from the transaction input")]
    DeployCallDecode,
    #[error("Invalid init data: Constructor not called")]
    InvalidInitData,
    #[error("Invalid deployer address")]
    InvalidDeployerAddress,
    #[error("Transaction receipt error")]
    TransactionReceiptError,
    #[error("Deployment transaction not successful")]
    TxNotSuccessful,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build local fragments (prefixing each chunk) from raw wasm chunks.
    fn local_fragments(chunks: &[&[u8]]) -> Vec<CodeFragment> {
        chunks
            .iter()
            .map(|chunk| CodeFragment::new(chunk))
            .collect()
    }

    /// The on-chain code deployed at each fragment address equals the fragment bytes verbatim.
    fn onchain_code(fragments: &[CodeFragment]) -> Vec<Bytes> {
        fragments
            .iter()
            .map(|fragment| Bytes::copy_from_slice(fragment.as_slice()))
            .collect()
    }

    fn addresses(count: usize) -> Vec<Address> {
        (0..count)
            .map(|i| Address::with_last_byte(i as u8))
            .collect()
    }

    #[test]
    fn matching_fragments_succeed() {
        let local = local_fragments(&[b"alpha", b"beta", b"gamma"]);
        let code = onchain_code(&local);
        let addrs = addresses(local.len());
        assert!(matches!(
            compare_fragments(100, &local, 100, &addrs, &code),
            VerificationStatus::Success
        ));
    }

    #[test]
    fn size_mismatch_fails() {
        let local = local_fragments(&[b"alpha"]);
        let code = onchain_code(&local);
        let addrs = addresses(local.len());
        assert!(matches!(
            compare_fragments(100, &local, 101, &addrs, &code),
            VerificationStatus::Failure(VerificationFailure::SizeMismatch {
                tx: 101,
                build: 100
            })
        ));
    }

    #[test]
    fn fragment_count_mismatch_fails() {
        let local = local_fragments(&[b"alpha", b"beta"]);
        let code = onchain_code(&local[..1]);
        let addrs = addresses(1);
        assert!(matches!(
            compare_fragments(100, &local, 100, &addrs, &code),
            VerificationStatus::Failure(VerificationFailure::FragmentCountMismatch {
                tx: 1,
                build: 2
            })
        ));
    }

    #[test]
    fn fragment_byte_mismatch_fails() {
        let local = local_fragments(&[b"alpha", b"beta"]);
        let mut code = onchain_code(&local);
        let mut tampered = code[1].to_vec();
        *tampered.last_mut().unwrap() ^= 0xff;
        code[1] = Bytes::from(tampered);
        let addrs = addresses(local.len());
        match compare_fragments(100, &local, 100, &addrs, &code) {
            VerificationStatus::Failure(VerificationFailure::FragmentMismatch {
                index,
                missing,
                ..
            }) => {
                assert_eq!(index, 1);
                assert!(!missing);
            }
            other => panic!("expected FragmentMismatch, got {other:?}"),
        }
    }

    #[test]
    fn missing_fragment_code_fails() {
        let local = local_fragments(&[b"alpha", b"beta"]);
        let mut code = onchain_code(&local);
        code[0] = Bytes::new();
        let addrs = addresses(local.len());
        match compare_fragments(100, &local, 100, &addrs, &code) {
            VerificationStatus::Failure(VerificationFailure::FragmentMismatch {
                index,
                missing,
                ..
            }) => {
                assert_eq!(index, 0);
                assert!(missing);
            }
            other => panic!("expected missing FragmentMismatch, got {other:?}"),
        }
    }
}
