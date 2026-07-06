// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::fmt;

use alloy::{
    consensus::Transaction,
    primitives::{Address, Bytes, TxHash},
    providers::Provider,
    rpc::types::TransactionReceipt,
    sol_types::SolCall,
};

use crate::{
    core::{
        code::{contract::ContractCode, fragments::CodeFragments, Code},
        deployment::{
            deployer::{
                get_address_from_receipt, stylus_constructorCall, StylusDeployer::deployCall,
                ADDRESS,
            },
            prelude::{DeploymentCalldata, PRELUDE_LENGTH},
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
    let receipt = provider
        .get_transaction_receipt(tx_hash)
        .await?
        .ok_or(TransactionReceiptError)?;
    if !receipt.status() {
        return Err(TxNotSuccessful);
    }

    // Rebuild the contract locally and compare it against the code the deployment installed
    // on-chain. A single-chunk contract is verified against the deployment calldata (prelude
    // included); a fragmented contract is verified against the root contract actually deployed at
    // the contract address.
    // Rebuild with the default check config, matching deploy: neither queries the chain's actual
    // `max_code_size` and both assume `DEFAULT_MAX_CODE_SIZE` for now, so local fragmentation here
    // reproduces the deployment's. On a chain with a non-default max code size this rebuild must
    // use that value (as the deployment would have), or fragmented contracts will chunk
    // differently and mis-verify.
    let status = contract.check(None, &Default::default(), provider).await?;

    match status.code() {
        Code::Contract(contract) => {
            let onchain = extract_deployment_calldata(&tx)?;
            Ok(compare_calldata(
                &onchain,
                &DeploymentCalldata::new(contract.as_slice()),
            ))
        }
        Code::Fragments(fragments) => {
            let address = deployed_address(&tx, &receipt)?;
            verify_fragments(address, fragments, provider).await
        }
    }
}

/// Resolve the address at which a deployment transaction installed its contract.
///
/// A plain CREATE transaction records the address in the receipt directly; a deployment routed
/// through the [`StylusDeployer`](deployCall) emits it in a `ContractDeployed` event.
fn deployed_address(
    tx: &impl Transaction,
    receipt: &TransactionReceipt,
) -> Result<Address, VerificationError> {
    match tx.to() {
        None => receipt
            .contract_address
            .ok_or(VerificationError::NoContractAddress),
        Some(_) => Ok(get_address_from_receipt(receipt)?),
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
    let calldata = match tx.to() {
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
            DeploymentCalldata(deploy_call.bytecode.to_vec())
        }
        None => DeploymentCalldata(tx.input().to_vec()),
    };
    // The calldata comes from untrusted transaction bytes; guard the fixed-offset prelude/wasm
    // accessors against input too short to contain a prelude (they would otherwise panic).
    if calldata.0.len() < PRELUDE_LENGTH {
        return Err(VerificationError::CalldataTooShort);
    }
    Ok(calldata)
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

/// Verify a fragmented deployment against the local build.
///
/// Reads the root contract actually deployed at `address` (rather than trusting the transaction's
/// claimed payload, which a non-standard prelude could contradict), fetches the code at each
/// fragment address it points to, and compares it against the locally built fragments.
async fn verify_fragments(
    address: Address,
    local: &CodeFragments,
    provider: &impl Provider,
) -> Result<VerificationStatus, VerificationError> {
    let root_code = provider.get_code_at(address).await?;
    let (onchain_uncompressed_size, addresses) = match ContractCode::parse_root_contract(&root_code)
    {
        Ok(parsed) => parsed,
        // Local build fragmented but the deployed code isn't a root contract
        Err(_) => {
            return Ok(VerificationStatus::Failure(
                VerificationFailure::FragmentationMismatch,
            ))
        }
    };

    let mut fragments = Vec::with_capacity(addresses.len());
    for address in addresses {
        let code = provider.get_code_at(address).await?;
        fragments.push((address, code));
    }

    Ok(compare_fragments(
        local,
        &OnchainFragments {
            uncompressed_wasm_size: onchain_uncompressed_size,
            fragments,
        },
    ))
}

/// A fragmented deployment as read from on-chain: the uncompressed wasm size recorded in the root
/// contract, paired with each fragment's address and the code deployed there.
struct OnchainFragments {
    uncompressed_wasm_size: u32,
    fragments: Vec<(Address, Bytes)>,
}

/// Compare a locally built set of fragments against the code read from a fragmented deployment.
fn compare_fragments(local: &CodeFragments, onchain: &OnchainFragments) -> VerificationStatus {
    if onchain.uncompressed_wasm_size != local.uncompressed_wasm_size() {
        return VerificationStatus::Failure(VerificationFailure::SizeMismatch {
            tx: onchain.uncompressed_wasm_size,
            build: local.uncompressed_wasm_size(),
        });
    }

    if onchain.fragments.len() != local.fragment_count() {
        return VerificationStatus::Failure(VerificationFailure::FragmentCountMismatch {
            tx: onchain.fragments.len(),
            build: local.fragment_count(),
        });
    }

    for (index, ((address, onchain_code), local_fragment)) in
        onchain.fragments.iter().zip(local.as_slice()).enumerate()
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
    /// The local build fragmented, but the on-chain deployment is not a fragmented contract.
    FragmentationMismatch,
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
            Self::FragmentationMismatch => {
                write!(
                    f,
                    "local build is fragmented but the on-chain deployment is not a fragmented \
                     contract (check that the toolchain and max code size match the deployment)"
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
    #[error("{0}")]
    Deployment(#[from] crate::core::deployment::DeploymentError),
    #[error("No code at address")]
    NoCodeAtAddress,
    #[error("Deployment transaction receipt has no contract address")]
    NoContractAddress,
    #[error("Deployment calldata is too short to contain a prelude")]
    CalldataTooShort,
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
    use crate::core::code::fragments::CodeFragment;

    /// Build local fragments (prefixing each chunk) from raw wasm chunks.
    fn local_fragments(uncompressed_wasm_size: u32, chunks: &[&[u8]]) -> CodeFragments {
        let fragments = chunks
            .iter()
            .map(|chunk| CodeFragment::new(chunk))
            .collect();
        CodeFragments::from_fragments(uncompressed_wasm_size, fragments)
    }

    /// On-chain fragments whose deployed code equals each local fragment's bytes verbatim.
    fn matching_onchain(uncompressed_wasm_size: u32, local: &CodeFragments) -> OnchainFragments {
        let fragments = local
            .as_slice()
            .iter()
            .enumerate()
            .map(|(i, fragment)| {
                (
                    Address::with_last_byte(i as u8),
                    Bytes::copy_from_slice(fragment.as_slice()),
                )
            })
            .collect();
        OnchainFragments {
            uncompressed_wasm_size,
            fragments,
        }
    }

    #[test]
    fn matching_fragments_succeed() {
        let local = local_fragments(100, &[b"alpha", b"beta", b"gamma"]);
        let onchain = matching_onchain(100, &local);
        assert!(matches!(
            compare_fragments(&local, &onchain),
            VerificationStatus::Success
        ));
    }

    #[test]
    fn size_mismatch_fails() {
        let local = local_fragments(100, &[b"alpha"]);
        let onchain = matching_onchain(101, &local);
        assert!(matches!(
            compare_fragments(&local, &onchain),
            VerificationStatus::Failure(VerificationFailure::SizeMismatch {
                tx: 101,
                build: 100
            })
        ));
    }

    #[test]
    fn fragment_count_mismatch_fails() {
        let local = local_fragments(100, &[b"alpha", b"beta"]);
        let mut onchain = matching_onchain(100, &local);
        onchain.fragments.truncate(1);
        assert!(matches!(
            compare_fragments(&local, &onchain),
            VerificationStatus::Failure(VerificationFailure::FragmentCountMismatch {
                tx: 1,
                build: 2
            })
        ));
    }

    #[test]
    fn fragment_byte_mismatch_fails() {
        let local = local_fragments(100, &[b"alpha", b"beta"]);
        let mut onchain = matching_onchain(100, &local);
        let mut tampered = onchain.fragments[1].1.to_vec();
        *tampered.last_mut().unwrap() ^= 0xff;
        onchain.fragments[1].1 = Bytes::from(tampered);
        match compare_fragments(&local, &onchain) {
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
        let local = local_fragments(100, &[b"alpha", b"beta"]);
        let mut onchain = matching_onchain(100, &local);
        onchain.fragments[0].1 = Bytes::new();
        match compare_fragments(&local, &onchain) {
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
