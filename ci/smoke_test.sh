#!/bin/bash

set -euo pipefail

# Print version information
rustc -Vv
cargo -V
cargo stylus --version

# Deploy ERC20 Example
# cd ./examples/erc20
# cargo stylus deploy -e http://localhost:8547 --private-key 0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659