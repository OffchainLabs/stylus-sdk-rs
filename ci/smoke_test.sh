#!/bin/bash

set -euo pipefail

# Print version information
rustc -Vv
cargo -V
#cargo stylus --version

cargo stylus new counter
cd counter
echo "[workspace]" >> Cargo.toml

cargo stylus deploy --private-key $PRIV_KEY -e https://stylus-testnet.arbitrum.io/rpc
