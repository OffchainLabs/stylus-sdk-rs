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

mod bytecode {
    pub const CACHE_MANAGER: &str = "60a06040523060805234801561001457600080fd5b50608051611d1c61003060003960006105260152611d1c6000f3fe";
    pub const STYLUS_DEPLOYER: &str = "608060405234801561001057600080fd5b506108a8806100206000396000f3fe6080604052600436106100345760003560e01c8063835d1d4c146100395780639f40b3851461006e578063a9a8e4e91461009c575b600080fd5b34801561004557600080fd5b50610059610054366004610612565b6100d4565b60405190151581526020015b60405180910390f35b34801561007a57600080fd5b5061008e610089366004610691565b6101f0565b604051908152602001610065565b6100af6100aa3660046106dd565b610226565b60405173ffffffffffffffffffffffffffffffffffffffff9091168152602001610065565b6040517fd70c0ca700000000000000000000000000000000000000000000000000000000815273ffffffffffffffffffffffffffffffffffffffff82163f6004820152600090819060719063d70c0ca790602401602060405180830381865afa925050508015610161575060408051601f3d908101601f1916820190925261015e91810190610772565b60015b61016d57506000610170565b90505b607173ffffffffffffffffffffffffffffffffffffffff1663a996e0c26040518163ffffffff1660e01b8152600401602060405180830381865afa1580156101bc573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906101e09190610772565b61ffff9182169116141592915050565b600083838360405160200161020793929190610794565b6040516020818303038152906040528051906020012090509392505050565b6000811561023c576102398286866101f0565b91505b600061027f88888080601f016020809104026020016040519081016040528093929190818152602001838380828437600092019190915250879250610541915050565b9050600061028c826100d4565b90506000811561033c5760006102a287346107ae565b6040517f58c780c200000000000000000000000000000000000000000000000000000000815273ffffffffffffffffffffffffffffffffffffffff861660048201529091506071906358c780c2908390602401604080518083038185885af1158015610312573d6000803e3d6000fd5b50505050506040513d601f19601f8201168201806040525081019061033791906107e8565b925050505b861561040c5760008373ffffffffffffffffffffffffffffffffffffffff16878a8a60405161036c929190610814565b60006040518083038185875af1925050503d80600081146103a9576040519150601f19603f3d011682016040523d82523d6000602084013e6103ae565b606091505b5050905080610406576040517fb66f7a3600000000000000000000000000000000000000000000000000000000815273ffffffffffffffffffffffffffffffffffffffff851660048201526024015b60405180910390fd5b50610443565b8515610443576040517ecc797100000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b60008661045083346107ae565b61045a91906107ae565b905080156104e957604051600090339083908381818185875af1925050503d80600081146104a4576040519150601f19603f3d011682016040523d82523d6000602084013e6104a9565b606091505b50509050806104e7576040517f3ea99169000000000000000000000000000000000000000000000000000000008152600481018390526024016103fd565b505b60405173ffffffffffffffffffffffffffffffffffffffff851681527f8ffcdc15a283d706d38281f500270d8b5a656918f555de0913d7455e3e6bc1bf9060200160405180910390a150919998505050505050505050565b6000825160000361057e576040517f21744a5900000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b6000821561059757828451602086016000f590506105a3565b8351602085016000f090505b3d1519811516156105ba576040513d6000823e3d81fd5b73ffffffffffffffffffffffffffffffffffffffff811661060957836040517f794c92ce0000000000000000000000000000000000000000000000000000000081526004016103fd9190610824565b90505b92915050565b60006020828403121561062457600080fd5b813573ffffffffffffffffffffffffffffffffffffffff8116811461060957600080fd5b60008083601f84011261065a57600080fd5b50813567ffffffffffffffff81111561067257600080fd5b60208301915083602082850101111561068a57600080fd5b9250929050565b6000806000604084860312156106a657600080fd5b83359250602084013567ffffffffffffffff8111156106c457600080fd5b6106d086828701610648565b9497909650939450505050565b600080600080600080608087890312156106f657600080fd5b863567ffffffffffffffff8082111561070e57600080fd5b61071a8a838b01610648565b9098509650602089013591508082111561073357600080fd5b5061074089828a01610648565b979a9699509760408101359660609091013595509350505050565b805161ffff8116811461076d57600080fd5b919050565b60006020828403121561078457600080fd5b61078d8261075b565b9392505050565b838152818360208301376000910160200190815292915050565b8181038181111561060c577f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b600080604083850312156107fb57600080fd5b6108048361075b565b9150602083015190509250929050565b8183823760009101908152919050565b600060208083528351808285015260005b8181101561085157858101830151858201604001528201610835565b506000604082860101526040601f19601f830116850101925050509291505056fea26469706673582212204448eb93d09c22a334820f79d39cc9a956fda57bbb8faa630df7c3446577503064736f6c63430008110033";
    pub const CREATE2_FACTORY_RAW_TX: &str = "f8a58085174876e800830186a08080b853604580600e600039806000f350fe7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe03601600081602082378035828234f58015156039578182fd5b8082525050506014600cf31ba02222222222222222222222222222222222222222222222222222222222222222a02222222222222222222222222222222222222222222222222222222222222222";
}

pub mod addresses {
    pub use alloy::primitives::{address, Address};

    pub const ARB_DEBUG: Address = address!("0x00000000000000000000000000000000000000FF");
    pub const ARB_OWNER: Address = address!("0x0000000000000000000000000000000000000070");
    pub const CREATE2_FACTORY: Address = address!("0x4e59b44847b379578588920ca78fbf26c0b4956c");
    pub const STYLUS_DEPLOYER: Address = address!("0x6ac4839Bfe169CadBBFbDE3f29bd8459037Bf64e");
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
        assert_eq!(stylus_deployer_code.len(), 2216);
        Ok(())
    }
}
