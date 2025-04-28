#!/bin/bash

set -euo pipefail

export RUSTFLAGS="-D warnings"
export RUSTFMT_CI=1

# Print version information
rustc -Vv
cargo -V

# Build and test main crate
if [ "$CFG_RELEASE_CHANNEL" == "nightly" ]; then
    echo "nightly build"
    cargo build --locked --all-features
else
    echo "regular build"
    cargo build --locked
fi
echo "running tests"
cargo test --all-features
