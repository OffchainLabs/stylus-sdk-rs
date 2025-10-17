# Cargo Stylus

A cargo subcommand for building, verifying, and deploying Arbitrum Stylus WASM contracts in Rust.

> [!NOTE]  
> Stylus contract verification will only be supported on Arbiscan for contracts deployed using cargo stylus `v0.5.0` or higher. We highly recommend deploying on Arbitrum Sepolia and verify your contracts on Sepolia Arbiscan first before going to mainnet.

## Table of Contents

- [Quick Start](#quick-start)
  - [Quick Usage](#quick-usage)
  - [Installing With Cargo](#installing-with-cargo)
  - [Building the Project Locally](#building-the-project-locally)
  - [Overview](#overview)
- [Deploying Non-Rust WASM Projects](#deploying-non-rust-wasm-projects)
- [Exporting Solidity ABIs](#exporting-solidity-abis)
- [Optimizing Binary Sizes](#optimizing-binary-sizes)
- [Command Reference](#command-reference)
  - [cargo stylus new](#cargo-stylus-new)
  - [cargo stylus check](#cargo-stylus-check)
  - [cargo stylus deploy](#cargo-stylus-deploy)
  - [cargo stylus verify](#cargo-stylus-verify)
  - [cargo stylus export-abi](#cargo-stylus-export-abi)
- [Troubleshooting](#troubleshooting)
  - [Common Issues and Solutions](#common-issues-and-solutions)
  - [Getting Additional Help](#getting-additional-help)
- [License](#license)

## Quick Start

![Image](./header.png)

### Quick Usage

```bash
# Install cargo-stylus
cargo install cargo-stylus
rustup target add wasm32-unknown-unknown

# Create a new project
cargo stylus new my-contract

# Check if contract can be deployed
cargo stylus check

# Deploy to testnet (Arbitrum Sepolia)
cargo stylus deploy --private-key-path=/path/to/key.txt --endpoint="https://sepolia-rollup.arbitrum.io/rpc"

# Export contract ABI
cargo stylus export-abi --output=./abi.json --json
```

### Installing With Cargo

Install [Rust](https://www.rust-lang.org/tools/install), and then install the plugin using the Cargo tool:

```shell
cargo install cargo-stylus
```

Add the `wasm32-unknown-unknown` build target to your Rust compiler:

```
rustup target add wasm32-unknown-unknown
```

You should now have it available as a Cargo subcommand:

```shell
cargo stylus --help

Cargo command for developing Arbitrum Stylus projects
```

### Building the Project Locally

Install [Rust](https://www.rust-lang.org/tools/install)

Clone the latest [released version](https://github.com/OffchainLabs/stylus-sdk-rs/releases) to your local device.

```shell
git clone --branch [VERSION_TAG] https://github.com/OffchainLabs/stylus-sdk-rs.git
cd stylus-sdk-rs/cargo-stylus
```

Run the `install.sh` script to build and install the local binaries to cargo

```shell
./install.sh
```

Add the `wasm32-unknown-unknown` build target to your Rust compiler:

```shell
rustup target add wasm32-unknown-unknown
```

When testing changes to your local repository, ensure that commands such as `cargo stylus deploy` are run with the `--no-verify` flag to opt out of using Docker

If your changes are localized to a single package, you can avoid building and reinstalling all packages by running

```shell
cargo install --path <workspace_pkg_with_changes>
```

### Overview

The cargo stylus command comes with useful commands such as `new`, `check` and `deploy`, and `export-abi` for developing and deploying Stylus contracts to Arbitrum chains. Here's a common workflow:

Start a new Stylus project with

```shell
cargo stylus new <YOUR_PROJECT_NAME>
```

The command above clones a local copy of the [stylus-hello-world](https://github.com/OffchainLabs/stylus-hello-world) starter project, which implements a Counter smart contract in Rust. See the [README](https://github.com/OffchainLabs/stylus-hello-world/blob/main/README.md) of stylus-hello-world for more details.

You can also use `cargo stylus new --minimal <YOUR_PROJECT_NAME>` to create a more barebones example with a Stylus entrypoint locally.

### Testnet Information

All testnet information, including faucets and RPC endpoints can be found [here](https://docs.arbitrum.io/stylus/reference/testnet-information).

### Developing With Stylus

Then, develop your Rust contract normally and take advantage of all the features the [stylus-sdk](https://github.com/OffchainLabs/stylus-sdk-rs) has to offer. To check whether or not your contract will successfully deploy and activate onchain, use the `cargo stylus check` subcommand:

```shell
cargo stylus check
```

This command will attempt to verify that your contract can be deployed and activated onchain without requiring a transaction by specifying a JSON-RPC endpoint. By default, it will use the public URL of the Stylus testnet as its endpoint. See [here](https://docs.arbitrum.io/stylus/reference/testnet-information) for available testnet RPC URLs. See `cargo stylus check --help` for more options.

If the command above fails, you'll see detailed information about why your WASM will be rejected:

```shell
Reading WASM file at bad-export.wat
Compressed WASM size: 55 B
Stylus checks failed: contract predeployment check failed when checking against
ARB_WASM_ADDRESS 0x0000…0071: (code: -32000, message: contract activation failed: failed to parse contract)

Caused by:
    binary exports reserved symbol stylus_ink_left

Location:
    prover/src/binary.rs:493:9, data: None
```

To read more about what counts as valid vs. invalid user WASM contracts, see [VALID_WASM](./main/VALID_WASM.md).

If your contract succeeds, you'll see the following message:

```shell
Finished release [optimized] target(s) in 1.88s
Reading WASM file at hello-stylus/target/wasm32-unknown-unknown/release/hello-stylus.wasm
Compressed WASM size: 3 KB
Contract succeeded Stylus onchain activation checks with Stylus version: 1
```

Once you're ready to deploy your contract onchain, you can use the `cargo stylus deploy` subcommand as follows:

First, we can estimate the gas required to perform our deployment and activation with:

```shell
cargo stylus deploy \
  --private-key-path=<PRIVKEY_FILE_PATH> \
  --estimate-gas
```

and see:

```shell
Compressed WASM size: 3 KB
Deploying contract to address 0x457b1ba688e9854bdbed2f473f7510c476a3da09
Estimated gas: 12756792
```

Next, attempt an actual deployment. Two transactions will be sent onchain.

```shell
cargo stylus deploy \
  --private-key-path=<PRIVKEY_FILE_PATH>
```

and see:

```shell
Compressed WASM size: 3 KB
Deploying contract to address 0x457b1ba688e9854bdbed2f473f7510c476a3da09
Estimated gas: 12756792
Submitting tx...
Confirmed tx 0x42db…7311, gas used 11657164
Activating contract at address 0x457b1ba688e9854bdbed2f473f7510c476a3da09
Estimated gas: 14251759
Submitting tx...
Confirmed tx 0x0bdb…3307, gas used 14204908
```

## Compiling and Checking Stylus Contracts

**cargo stylus check**

Instruments a Rust project using Stylus. This command runs compiled WASM code through Stylus instrumentation checks and reports any failures. It **verifies the contract can compile onchain** by making an eth_call to a Arbitrum chain RPC endpoint.

Usage: `cargo stylus check [OPTIONS]`

See `--help` for all available flags and default values.

## Deploying Stylus Contracts

**cargo stylus deploy**

Instruments a Rust project using Stylus and by outputting its brotli-compressed WASM code. Then, it submits **two transactions** by default: the first **deploys** the WASM contract code to an address and the second triggers an **activation onchain**. Developers can choose to split up the deploy and activate steps via this command as desired.

Usage: `cargo stylus deploy [OPTIONS]`

See `--help` for all available flags and default values.

## Verifying Stylus Contracts

See the formal Arbitrum docs on verifying Stylus contracts [here](https://docs.arbitrum.io/stylus/how-tos/verifying-contracts#reproducible-verification)

## Deploying Non-Rust WASM Projects

The Stylus tool can also be used to deploy non-Rust, WASM projects to Stylus by specifying the WASM file directly with the `--wasm-file` flag to any of the cargo stylus commands.

Even WebAssembly Text [(WAT)](https://www.webassemblyman.com/wat_webassembly_text_format.html) files are supported. This means projects that are just individual WASM files can be deployed onchain without needing to have been compiled by Rust. WASMs produced by other languages, such as C, can be used with the tool this way.

For example:

```js
(module
    (memory 0 0)
    (export "memory" (memory 0))
    (type $t0 (func (param i32) (result i32)))
    (func $add_one (export "add_one") (type $t0) (param $p0 i32) (result i32)
        local.get $p0
        i32.const 1
        i32.add)
    (func (export "user_entrypoint") (param $args_len i32) (result i32)
        (i32.const 0)
    ))
```

can be saved as `add.wat` and used as `cargo stylus check --wasm-file=add.wat` or `cargo stylus deploy --wasm-file=add.wat`.

## Exporting Solidity ABIs

Stylus Rust projects that use the [stylus-sdk](https://github.com/OffchainLabs/stylus-sdk-rs) have the option of exporting Solidity ABIs. The cargo stylus tool also makes this easy with the `export-abi` command:

```shell
cargo stylus export-abi
```

## Optimizing Binary Sizes

Brotli-compressed, Stylus contract WASM binaries must fit within the **24Kb** [code-size limit](https://ethereum.org/en/developers/tutorials/downsizing-contracts-to-fight-the-contract-size-limit/) of Ethereum smart contracts. By default, the `cargo stylus check` will attempt to compile a Rust contract into WASM with reasonable optimizations and verify its compressed size fits within the limit. However, there are additional options available in case a contract exceeds the 24Kb limit from using default settings. Deploying smaller binaries onchain is cheaper and better for the overall network, as deployed WASM contracts will exist on the Arbitrum chain's storage forever.

We recommend optimizing your Stylus contract's sizes to smaller sizes, but keep in mind the safety tradeoffs of using some of the more advanced optimizations. However, some small contracts when compiled to much smaller sizes can suffer performance penalties.

For a deep-dive into the different options for optimizing binary sizes using cargo stylus, see [OPTIMIZING_BINARIES.md](./main/OPTIMIZING_BINARIES.md).

## Command Reference

Below are the major commands with their syntax, common options, and examples:

### cargo stylus new

Creates a new Stylus project from a template.

**Syntax:**

```shell
cargo stylus new [OPTIONS]
```

**Common Options:**

- `--minimal`: Create a minimal project structure rather than the full example

**Examples:**

```shell
# Create a full-featured Counter example project
cargo stylus new my-token-contract

# Create a minimal project with just the essentials
cargo stylus new --minimal minimal-contract
```

### cargo stylus check

Validates that a contract can be deployed and activated on Arbitrum Stylus.

**Syntax:**

```shell
cargo stylus check [OPTIONS]
```

**Common Options:**

- `--endpoint=<URL>:` Arbitrum RPC endpoint (default: Arbitrum Sepolia)
- `--wasm-file=<PATH>`: Path to WASM file (if not using current project)
- `--contract-address=<ADDRESS>`: Target contract address (default: random address)

**Examples:**

```shell
# Check the current project against default testnet
cargo stylus check

# Check against a specific network
cargo stylus check --endpoint="https://arb1.arbitrum.io/rpc"

# Check a specific WASM file
cargo stylus check --wasm-file=./path/to/contract.wasm
```

### cargo stylus deploy

Deploys a Stylus contract to an Arbitrum chain and activates it.

**Syntax:**

```shell
cargo stylus deploy [OPTIONS]
```

**Common Options:**

- `--endpoint=<URL>`: Arbitrum RPC endpoint (default: Arbitrum Sepolia)
- `--private-key-path=<PATH>`: Path to file containing private key
- `--estimate-gas`: Only estimate the gas needed for deployment
- `--no-verify`: Skip using Docker for reproducible builds
- `--no-activate`: Deploy without activating the contract

**Examples:**

```shell
# Deploy to default testnet and estimate gas
cargo stylus deploy --private-key-path=./key.txt --estimate-gas

# Deploy to mainnet without Docker verification
cargo stylus deploy --endpoint="https://arb1.arbitrum.io/rpc" \
  --private-key-path=./key.txt --no-verify

# Deploy without activation (for advanced use cases)
cargo stylus deploy --private-key-path=./key.txt --no-activate
```

### cargo stylus verify

Verifies a previously deployed contract.

**Syntax:**

```shell
cargo stylus verify [OPTIONS]
```

Common Options:

- `--endpoint=<URL>`: Arbitrum RPC endpoint (default: Arbitrum Sepolia)
- `--deployment-tx=<TX_HASH>`: Hash of the deployment transaction
- `--no-verify`: Skip using Docker for reproducible builds

**Examples:**

```shell
# Verify a contract on Arbitrum Sepolia
  cargo stylus verify --deployment-tx=0x1234abcd...

# Verify a contract on mainnet
cargo stylus verify --endpoint="https://arb1.arbitrum.io/rpc" \
    --deployment-tx=0x5678efgh...
```

### cargo stylus export-abi

Exports a Solidity ABI for the current project.

**Syntax:**

```shell
cargo stylus export-abi [OPTIONS]
```

Common Options:

- `--output=<PATH>`: Output file path (default: stdout)
- `--json`: Generate JSON format ABI (requires solc)
- `--rust-features`=<FEATURES>: Rust features to include

**Examples:**

```shell
# Export ABI to stdout

cargo stylus export-abi

# Export JSON ABI to file
cargo stylus export-abi --output=./abi.json --json
  # Export with specific Rust features
cargo stylus export-abi --rust-features=feature1,feature2
```

## Troubleshooting

### Common Issues and Solutions

#### Size Limit Errors

**Error**: Contract exceeds 24KB after compression
**Solution**:

- Check [OPTIMIZING_BINARIES.md](./main/OPTIMIZING_BINARIES.md) for optimization techniques
- Use `#[no_std]` to avoid the Rust standard library
- Remove unused dependencies from your `Cargo.toml`
- Use the `opt-level = "z"` optimization in your release profile

#### WASM Validation Errors

**Error**: Contract fails with "binary exports reserved symbol" or other validation errors
**Solution**:

- Ensure you're using a supported version of the stylus-sdk
- Check [VALID_WASM.md](./main/VALID_WASM.md) for limitations on WASM features
- Make sure your contract properly exports the required entrypoint functions

#### Activation Failures

**Error**: Contract deployment succeeds but activation fails
**Solution**:

- Verify that your WASM contract is valid with `cargo stylus check`
- Ensure you have sufficient funds for both deployment and activation
- Check that your contract doesn't exceed size or feature limitations

### Getting Additional Help

If you encounter issues not covered here:

- File an issue in the [GitHub repository](https://github.com/OffchainLabs/stylus-sdk-rs/issues)
- Join the [Arbitrum Discord](https://discord.gg/arbitrum) for community support

## License

Cargo Stylus is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

