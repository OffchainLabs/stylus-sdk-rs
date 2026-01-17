// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod integration_test {
    use alloy::{primitives::U256, sol};
    use erc20::{erc20::Erc20Params, StylusTestTokenParams};
    use eyre::Result;
    use stylus_tools::devnet::addresses::OWNER;
    use stylus_tools::utils::testing::init_test;

    sol! {
        #[sol(rpc)]
        interface IStylusTestToken is IErc20  {
            function name() external pure returns (string memory);
            function symbol() external pure returns (string memory);
            function decimals() external pure returns (uint8);
            function totalSupply() external view returns (uint256);
            function balanceOf(address owner) external view returns (uint256);
            function transfer(address to, uint256 value) external returns (bool);
            function transferFrom(address from, address to, uint256 value) external returns (bool);
            function approve(address spender, uint256 value) external returns (bool);
            function allowance(address owner, address spender) external view returns (uint256);
            function mint(uint256 value) external;
            function mintTo(address to, uint256 value) external;
            function burn(uint256 value) external;
            error InsufficientBalance(address, uint256, uint256);
            error InsufficientAllowance(address, address, uint256, uint256);
        }
    }

    const EXPECTED_ABI: &str = "\
interface IStylusTestToken is IIErc20 {
    function mint(uint256 value) external;

    function mintTo(address to, uint256 value) external;

    function burn(uint256 value) external;

    error InsufficientBalance(address, uint256, uint256);

    error InsufficientAllowance(address, address, uint256, uint256);
}
interface IIErc20 {
    function name() external view returns (string memory);

    function symbol() external view returns (string memory);

    function decimals() external view returns (uint8);

    function totalSupply() external view returns (uint256);

    function balanceOf(address owner) external view returns (uint256);

    function transfer(address to, uint256 value) external returns (bool);

    function transferFrom(address from, address to, uint256 value) external returns (bool);

    function approve(address spender, uint256 value) external returns (bool);

    function allowance(address owner, address spender) external view returns (uint256);

    error InsufficientBalance(address, uint256, uint256);

    error InsufficientAllowance(address, address, uint256, uint256);
}";

    #[tokio::test]
    async fn erc20() -> Result<()> {
        let (devnode, address) = init_test(EXPECTED_ABI).await?;
        let provider = devnode.create_provider().await?;

        // Instantiate contract
        let contract = IStylusTestToken::IStylusTestTokenInstance::new(address, provider);

        // Check name
        let name = contract.name().call().await?;
        assert_eq!(name, StylusTestTokenParams::NAME);
        println!("ERC20.name(): {name}");

        // Check symbol
        let symbol = contract.symbol().call().await?;
        assert_eq!(symbol, StylusTestTokenParams::SYMBOL);
        println!("ERC20.symbol(): {symbol}");

        // Mint tokens
        let num_tokens = U256::from(1000);
        contract.mint(num_tokens).send().await?.watch().await?;

        // Check balance
        let balance = contract.balanceOf(OWNER).call().await?;
        assert_eq!(balance, num_tokens);

        Ok(())
    }
}
