// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::{
    network::{EthereumWallet, TransactionBuilder},
    primitives::{address, Address},
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
    sol,
};
use eyre::{bail, eyre, Result, WrapErr};
use regex::Regex;
use reqwest::{header::HeaderValue, Method, Response};
use std::{process::Command, str::FromStr};
use testcontainers::{
    core::{wait::HttpWaitStrategy, IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage, ImageExt,
};

pub const DEVNET_PRIVATE_KEY: &str =
    "b6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659";

const NITRO_IMAGE_NAME: &str = "offchainlabs/nitro-node";
const NITRO_IMAGE_TAG: &str = "v3.2.1-d81324d";
const NITRO_PORT: u16 = 8547;

sol! {
    #[sol(rpc)]
    interface ArbDebug {
        function becomeChainOwner() external;
    }

    #[sol(rpc)]
    interface ArbOwner {
        function addWasmCacheManager(address manager) external;
    }
}

pub struct DevNode {
    _container: ContainerAsync<GenericImage>,
    rpc: String,
}

impl DevNode {
    /// Starts a new Nitro devnode in the background that can be used to deploy Stylus contracts.
    /// This node will be shutdown when this struct is dropped.
    pub async fn new() -> Result<Self> {
        let wait_strategy = HttpWaitStrategy::new("/")
            .with_port(NITRO_PORT.into())
            .with_method(Method::POST)
            .with_header("Content-Type", HeaderValue::from_static("application/json"))
            .with_body(r#"{"jsonrpc":"2.0","method":"net_version","params":[],"id":1}"#)
            .with_response_matcher_async(nitro_response_matcher);
        let container = GenericImage::new(NITRO_IMAGE_NAME, NITRO_IMAGE_TAG)
            .with_exposed_port(NITRO_PORT.tcp())
            .with_wait_for(WaitFor::Http(wait_strategy))
            .with_cmd(vec![
                "--dev",
                "--http.addr",
                "0.0.0.0",
                "--http.api=net,web3,eth,debug",
            ])
            .start()
            .await?;
        let port = container.get_host_port_ipv4(NITRO_PORT).await?;
        let rpc = format!("http://localhost:{port}");
        setup_devnode(&rpc).await?;
        Ok(DevNode {
            _container: container,
            rpc,
        })
    }

    /// Gets the Nitro node RPC.
    pub fn rpc(&self) -> &str {
        &self.rpc
    }

    /// Deploys the Stylus contract in the current directory using cargo-stylus.
    pub async fn deploy(&self) -> Result<Address> {
        let output = Command::new("cargo-stylus")
            .arg("_")
            .arg("deploy")
            .arg("--no-verify")
            .arg("--endpoint")
            .arg(self.rpc())
            .arg("--private-key")
            .arg(DEVNET_PRIVATE_KEY)
            .output()
            .wrap_err("failed to run cargo-stylus deploy")?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("cargo-stylus deploy failed:\n{}", stderr)
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stdout = remove_colors(&stdout);
        let re = Regex::new(r"deployed code at address: (0x[a-fA-F0-9]{40})").unwrap();
        let Some(captures) = re.captures(&stdout) else {
            bail!("address not found in cargo-stylus output:\n{stdout}");
        };
        let Some(address) = captures.get(1) else {
            bail!("could not capture address from cargo-stylus output:\n{stdout}");
        };
        let address = Address::from_str(address.as_str())?;
        Ok(address)
    }
}

async fn nitro_response_matcher(response: Response) -> bool {
    let Ok(t) = response.text().await else {
        return false;
    };
    t.contains("result")
}

/// Prepares the devnode for deploying a Stylus contract.
async fn setup_devnode(rpc: &str) -> Result<()> {
    let signer: PrivateKeySigner = DEVNET_PRIVATE_KEY.parse()?;
    let wallet = EthereumWallet::from(signer);
    let owner = wallet.default_signer().address();
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_builtin(rpc)
        .await?;

    // Make the caller chain owner
    let address = address!("0x00000000000000000000000000000000000000FF");
    ArbDebug::new(address, &provider)
        .becomeChainOwner()
        .from(owner)
        .send()
        .await?
        .watch()
        .await?;

    // Deploy Cache Manager Contract
    let bytecode = alloy::hex::decode(
        "60a06040523060805234801561001457600080fd5b50608051611d1c61003060003960006105260152611d1c6000f3fe"
    )?;
    let tx = TransactionRequest::default().with_deploy_code(bytecode);
    let receipt = provider.send_transaction(tx).await?.get_receipt().await?;
    let cache_manager_address = receipt
        .contract_address
        .ok_or_else(|| eyre!("failed to get cache manager address"))?;
    println!("Cache Manager contract deployed at address: {cache_manager_address}");

    // Register the deployed Cache Manager contract
    let address = address!("0x0000000000000000000000000000000000000070");
    ArbOwner::new(address, &provider)
        .addWasmCacheManager(cache_manager_address)
        .from(owner)
        .send()
        .await?
        .watch()
        .await?;

    println!("Nitro node ready");
    Ok(())
}

fn remove_colors(colored_text: &str) -> std::borrow::Cow<'_, str> {
    let ansi_escape = Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    ansi_escape.replace_all(colored_text, "")
}
