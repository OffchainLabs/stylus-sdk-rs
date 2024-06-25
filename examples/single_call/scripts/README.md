# Single Call Stylus Contract

The purpose of this simple example is to test external contracts by routing calldata through the `SingleCall` contract to a target contract. This can be useful for testing existing Solidity contracts in a Stylus context. As part of this example, we'll deploy a simple Solidity `Counter` contract, and test incrementing the `Counter` by executing via the `SingleCall` contract.

## Basic Requirements

You'll need a developer wallet funded with some Sepolia ETH on the Arbitrum Sepolia testnet.

### Public Faucets

- [Alchemy Arbitrum Sepolia Faucet](https://www.alchemy.com/faucets/arbitrum-sepolia)
- [Quicknode Arbitrum Sepolia Faucet](https://faucet.quicknode.com/arbitrum/sepolia)

Note: If you already have some Sepolia ETH, but it has not been bridged to the Arbitrum Sepolia layer 2, you can do so using [the official Arbitrum bridge](https://bridge.arbitrum.io/?destinationChain=arbitrum-sepolia&sourceChain=sepolia). This process can take up to 15 minutes.

## Deploy the Rust `SingleCall` Contract

The source code for `SingleCall` contract can be found in the `/src/lib.rs` file. It is a simple contract that contains a single `execute(address,bytes)(bytes)` function signature. It takes a target contract as its first parameter and any arbitrary `bytes` as its second parameter. It will forward the `bytes` you pass in to that contract, and return any `bytes` it receives back.

A default instance of the `SingleCall` contract has been deployed to `0x5856f06b2924733049d87c261aba77f1f10be2a8` on Arbitrum Sepolia.

If you need to deploy to another network, then from the root `single_call` directory call:

```
cargo stylus deploy -e https://sepolia-rollup.arbitrum.io/rpc --private-key-path=./.path_to_your_key_file
```

The `cargo stylus` CLI tool can be found at [cargo-stylus](https://github.com/OffchainLabs/cargo-stylus) if you do not already have it installed. It is not necessary for this demo, however, if you use the pre-deployed contract.

## Set Up Your Environment Variables

From your terminal, run:

```
cp .env.example .env
```

Now open the `.env` file. You will see that `RPC_URL`, `TX_EXPLORER_URL`, and `SINGLE_CALL_CONTRACT_ADDRESS` has already been populated. We will deploy the `Counter` contract in just a moment, but first, you'll need to populate your `PRIVATE_KEY`.

**NOTE: DO NOT use a personal account to deploy contracts. Always use a fresh developer wallet that does not contain any real assets for development purposes.**

Make sure the account you're using has Sepolia ETH on Arbitrum Sepolia. You can check your balance on [Sepolia Arbiscan](https://sepolia.arbiscan.io/).

## Deploy the Solidity Counter Contract

We'll be deploying the Solidity `Counter.sol` contract that can be found in the `external_contracts` folder. It is a simple contract that contains two methods: `setNumber(uint256)` and `increment()`, as well as a getter for the public `number()(uint256)` value.

You'll need a recent version of [Node.js](https://nodejs.org/en) and [Yarn](https://yarnpkg.com/) to run these scripts. First, install all dependencies by running:

```
yarn
```

Then deploy `Counter.sol` by running the `deploy_counter.js` script:

```
yarn hardhat ignition deploy ignition/modules/deploy_counter.js --network arb_sepolia
```

You should see in your console something like:

```
✔ Confirm deploy to network arb_sepolia (421614)? … yes
Compiled 1 Solidity file successfully (evm target: paris).
Hardhat Ignition 🚀

Deploying [ deploy_counter ]

Batch #1
  Executed deploy_counter#Counter

Batch #2
  Executed deploy_counter#Counter.setNumber

[ deploy_counter ] successfully deployed 🚀

Deployed Addresses

deploy_counter#Counter - 0xfDa82C11DF0Eb7490fFACC0652cbcC36D49327Bd
```

Take note of the `Deployed Addresses` value and go ahead and copy and paste that as your `COUNTER_CONTRACT_ADDRESS` variable in your `.env` file.

## Incrementing `Counter` via `SingleCall`

Now you're all set to increment the `Counter` via `SingleCall`. There is a script that shows how to do this in `/src/main.js`. To run the script, simply call:

```
yarn run increment
```

You should see console output similar to below:

```
Incrementing the Counter contract at 0xaAf6112301a19c90feFb251D0567610eA649752D via the SingleCall router at 0xb27fc74caf34c5c26d27a7654358017331330cee
Current count: 42
0xd09de08a
Transaction hash: 0x35c6d2ea3de188ed6bd5283c49d58cf89fc12e65cece9ad19a62e158e0bc944e
View tx on explorer: https://sepolia.arbiscan.io/tx/0x35c6d2ea3de188ed6bd5283c49d58cf89fc12e65cece9ad19a62e158e0bc944e
Updated count: 43
✨  Done in 5.48s.
```
