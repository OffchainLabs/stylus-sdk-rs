// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{primitives::U256, sol};
    use erc721::{erc721::Erc721Params, StylusTestNFTParams};
    use eyre::Result;
    use stylus_tools::devnet::{addresses::OWNER, Node};

    sol! {
        #[sol(rpc)]
        interface IStylusTestNFT {
            function name() external pure returns (string memory);
            function symbol() external pure returns (string memory);
            function tokenURI(uint256 token_id) external view returns (string memory);
            function balanceOf(address owner) external view returns (uint256);
            function ownerOf(uint256 token_id) external view returns (address);
            function safeTransferFrom(address from, address to, uint256 token_id, bytes calldata data) external;
            function safeTransferFrom(address from, address to, uint256 token_id) external;
            function transferFrom(address from, address to, uint256 token_id) external;
            function approve(address approved, uint256 token_id) external;
            function setApprovalForAll(address operator, bool approved) external;
            function getApproved(uint256 token_id) external returns (address);
            function isApprovedForAll(address owner, address operator) external returns (bool);
            function supportsInterface(bytes4 _interface) external pure returns (bool);
            function mint() external;
            function mintTo(address to) external;
            function burn(uint256 token_id) external;
            function totalSupply() external returns (uint256);
            error NotOwner(address, uint256, address);
            error NotApproved(address, address, uint256);
            error TransferToZero(uint256);
            error ReceiverRefused(address, uint256, bytes4);
            error InvalidTokenId(uint256);
        }
    }

    #[tokio::test]
    async fn erc721() -> Result<()> {
        let devnode = Node::new().await?;
        let rpc = devnode.rpc();
        println!("Deploying contract to Nitro ({rpc})...");
        let (address, _) = stylus_tools::Deployer::new(rpc.to_owned()).deploy()?;
        println!("Deployed contract to {address}");
        let provider = devnode.create_provider().await?;
        let contract = IStylusTestNFT::IStylusTestNFTInstance::new(address, provider);

        // Check name
        let name = contract.name().call().await?;
        assert_eq!(name, StylusTestNFTParams::NAME);
        println!("ERC721.name(): {name}");

        // Check symbol
        let symbol = contract.symbol().call().await?;
        assert_eq!(symbol, StylusTestNFTParams::SYMBOL);
        println!("ERC721.symbol(): {symbol}");

        // Mint NFTs
        const NUM_NFTS: usize = 3;
        for _ in 0..NUM_NFTS {
            contract.mint().send().await?.watch().await?;
        }

        // Check balance
        let balance = contract.balanceOf(OWNER).call().await?;
        assert_eq!(balance, U256::from(NUM_NFTS));

        Ok(())
    }
}
