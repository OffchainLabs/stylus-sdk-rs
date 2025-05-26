#!/bin/bash

set -euo pipefail

export RUSTFMT_CI=1

# Print version information
rustc -Vv
cargo -V

cargo test -p stylus-tools -F integration-tests

pushd examples/erc20
cargo check -F integration-tests --locked --all-targets
cargo test -F integration-tests
popd

pushd examples/erc721
cargo check -F integration-tests --locked --all-targets
cargo test -F integration-tests
popd

pushd examples/single_call
cargo check -F integration-tests --locked --all-targets
cargo test -F integration-tests
popd
