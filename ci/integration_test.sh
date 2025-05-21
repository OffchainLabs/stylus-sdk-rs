#!/bin/bash

set -euo pipefail

export RUSTFLAGS="-D warnings"
export RUSTFMT_CI=1

# Print version information
rustc -Vv
cargo -V

cargo test -p stylus-tools -F testcontainers

pushd examples/erc20
cargo check --locked
cargo test
popd
