#!/bin/bash

set -euo pipefail

# Print version information
rustc -Vv
cargo -V

REPO_ROOT=$(git rev-parse --show-toplevel)
TEST_DIR=$(mktemp -d)
echo "Running smoke test in isolated directory: $TEST_DIR"
cd "$TEST_DIR"

cargo stylus new counter --sdk-path "$REPO_ROOT/stylus-sdk"
cd counter

# Verify scaffolding is correct
echo "Verifying scaffolded project..."
test -f Cargo.lock || { echo "FAIL: Cargo.lock not generated"; exit 1; }
test -f Stylus.toml || { echo "FAIL: Stylus.toml not generated"; exit 1; }
test -f rust-toolchain.toml || { echo "FAIL: rust-toolchain.toml not generated"; exit 1; }
test -f src/lib.rs || { echo "FAIL: src/lib.rs not generated"; exit 1; }
test -f src/main.rs || { echo "FAIL: src/main.rs not generated"; exit 1; }
grep -q 'stylus-sdk' Cargo.toml || { echo "FAIL: stylus-sdk not in Cargo.toml"; exit 1; }
grep -q 'entrypoint' src/lib.rs || { echo "FAIL: src/lib.rs missing entrypoint"; exit 1; }
echo "Scaffolding OK"

# Add workspace key required for cargo stylus commands
echo "[workspace]" >> Cargo.toml

# Use the nitro testnode private key found from the public mnemonic
# https://github.com/OffchainLabs/nitro-testnode/blob/5986e62e8fc8672858baf0550443991adc23f9c2/scripts/consts.ts#L6
cargo stylus deploy --private-key 0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659 -e http://localhost:8547
