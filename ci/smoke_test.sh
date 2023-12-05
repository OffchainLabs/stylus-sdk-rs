#!/bin/bash

set -euo pipefail

# Print version information
rustc -Vv
cargo -V
cargo stylus --version

cargo stylus new counter
cd counter
echo "[workspace]" >> Cargo.toml

cargo stylus deploy -e http://localhost:8547 --private-key $PRIV_KEY
