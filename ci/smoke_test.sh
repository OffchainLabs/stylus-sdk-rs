#!/bin/bash

set -euo pipefail

# Print version information
rustc -Vv
cargo -V
#cargo stylus --version

REPO_ROOT=$(git rev-parse --show-toplevel)
TEST_DIR=$(mktemp -d)
echo "Running smoke test in isolated directory: $TEST_DIR"
cd "$TEST_DIR"

cargo stylus new counter
cd counter
cargo remove stylus-sdk
cargo add stylus-sdk --path "$REPO_ROOT/stylus-sdk"
echo "[workspace]" >> Cargo.toml

# Use the nitro testnode private key found from the public mnemonic
# https://github.com/OffchainLabs/nitro-testnode/blob/5986e62e8fc8672858baf0550443991adc23f9c2/scripts/consts.ts#L6
cargo stylus deploy --private-key 0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659 -e http://localhost:8547
