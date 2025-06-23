// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::{
    network::{EthereumWallet, TransactionBuilder},
    primitives::{address, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
    sol,
};
use eyre::{Result, WrapErr};
use reqwest::{header::HeaderValue, Method, Response};
use testcontainers::{
    core::{wait::HttpWaitStrategy, IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage, ImageExt,
};

pub const DEVNET_PRIVATE_KEY: &str =
    "b6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659";

const NITRO_IMAGE_NAME: &str = "offchainlabs/nitro-node";
const NITRO_IMAGE_TAG: &str = "v3.5.6-9a29a1e";
const NITRO_PORT: u16 = 8547;

mod bytecode;

pub mod addresses {
    pub use alloy::primitives::{address, Address};

    pub const OWNER: Address = address!("0x3f1Eae7D46d88F08fc2F8ed27FCb2AB183EB2d0E");
    pub const ARB_DEBUG: Address = address!("0x00000000000000000000000000000000000000FF");
    pub const ARB_OWNER: Address = address!("0x0000000000000000000000000000000000000070");
    pub const CREATE2_FACTORY: Address = address!("0x4e59b44847b379578588920ca78fbf26c0b4956c");
    pub const STYLUS_DEPLOYER: Address = address!("0xcEcba2F1DC234f70Dd89F2041029807F8D03A990");
    pub const CACHE_MANAGER: Address = address!("0x2F8Bd4EaB69764c105aDd7dE7CB0402557a44E6f");
}

sol! {
    #[sol(rpc)]
    interface ArbDebug {
        function becomeChainOwner() external;
    }

    #[sol(rpc)]
    interface ArbOwner {
        function addWasmCacheManager(address manager) external;
        function setL1PricePerUnit(uint256 value) external;
    }
}

/// Manage a devnet node for deploying Stylus contracts.
pub struct Node {
    _container: ContainerAsync<GenericImage>,
    rpc: String,
}

impl Node {
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
            .await
            .wrap_err("failed to start Nitro container")?;
        let port = container
            .get_host_port_ipv4(NITRO_PORT)
            .await
            .wrap_err("failed to get Nitro RPC port")?;
        let rpc = format!("http://localhost:{port}");
        let devnode = Node {
            _container: container,
            rpc,
        };
        devnode.setup().await?;
        Ok(devnode)
    }

    /// Get the Nitro node RPC.
    pub fn rpc(&self) -> &str {
        &self.rpc
    }

    /// Create a provider with the chain owner keys to send requests to the node.
    pub async fn create_provider(&self) -> Result<impl Provider> {
        let signer: PrivateKeySigner = DEVNET_PRIVATE_KEY
            .parse()
            .expect("failed to parse devnet private key");
        let wallet = EthereumWallet::from(signer);
        let provider = ProviderBuilder::new()
            .wallet(wallet)
            .connect(self.rpc())
            .await?;
        Ok(provider)
    }

    async fn setup(&self) -> Result<()> {
        let provider = self.create_provider().await?;

        // Make the caller chain owner
        ArbDebug::new(addresses::ARB_DEBUG, &provider)
            .becomeChainOwner()
            .send()
            .await?
            .watch()
            .await?;

        // Set the L1 data fee to 0 so it doesn't impact the L2 Gas limit.
        // This makes the gas estimates closer to Ethereum and allows the deployment of the CREATE2 factory.
        let arbowner = ArbOwner::new(addresses::ARB_OWNER, &provider);
        arbowner
            .setL1PricePerUnit(U256::ZERO)
            .send()
            .await?
            .watch()
            .await?;

        // Send funds to CREATE2 factory deployer
        let factory_deployer = address!("0x3fab184622dc19b6109349b94811493bf2a45362");
        let value = alloy::primitives::utils::parse_ether("0.1")?;
        let tx = TransactionRequest::default()
            .with_to(factory_deployer)
            .with_value(value);
        provider.send_transaction(tx).await?.watch().await?;

        // Deploy CREATE2 factory
        let factory_raw_tx = alloy::hex::decode(bytecode::CREATE2_FACTORY_RAW_TX)?;
        provider
            .send_raw_transaction(&factory_raw_tx)
            .await?
            .watch()
            .await?;

        // Deploy CacheManager contract
        let mut input = vec![0; 32];
        input.extend_from_slice(&alloy::hex::decode(bytecode::CACHE_MANAGER)?);
        let tx = TransactionRequest::default()
            .with_to(addresses::CREATE2_FACTORY)
            .with_input(input);
        provider.send_transaction(tx).await?.get_receipt().await?;

        // Register the deployed Cache Manager contract
        arbowner
            .addWasmCacheManager(addresses::CACHE_MANAGER)
            .send()
            .await?
            .watch()
            .await?;

        // Deploy StylusDeployer
        let mut input = vec![0; 32];
        input.extend_from_slice(&alloy::hex::decode(bytecode::STYLUS_DEPLOYER)?);
        let tx = TransactionRequest::default()
            .with_to(addresses::CREATE2_FACTORY)
            .with_input(input);
        provider.send_transaction(tx).await?.get_receipt().await?;

        Ok(())
    }
}

async fn nitro_response_matcher(response: Response) -> bool {
    let Ok(text) = response.text().await else {
        return false;
    };
    text.contains("result")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn node_deploys_contracts() -> Result<()> {
        let devnode = Node::new().await?;
        let provider = devnode.create_provider().await?;
        let cache_manager_code = provider.get_code_at(addresses::CACHE_MANAGER).await?;
        assert_eq!(cache_manager_code.len(), 7452);
        let stylus_deployer_code = provider.get_code_at(addresses::STYLUS_DEPLOYER).await?;
        assert_eq!(stylus_deployer_code.len(), 2269);
        Ok(())
    }
}
