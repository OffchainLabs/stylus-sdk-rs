#!/bin/bash

set -euo pipefail

export RUSTFLAGS="-D warnings"
export RUSTFMT_CI=1

# Print version information
rustc -Vv
cargo -V

# Build and test main crate
if [ "$CFG_RELEASE_CHANNEL" == "nightly" ]; then
    cargo build --locked --all-features
else
    cargo build --locked
fi

# Select all features but the integration test one, which will be run in another CI job.
# We have to use cargo metadata because we can't exclude a feature directly in cargo test.
# See: https://github.com/rust-lang/cargo/issues/3126
FEATURES=$(cargo metadata --format-version=1 --no-deps | jq -r '.packages[] | select(.name == "stylus-sdk") | .features | keys | map(select(. != "testcontainers")) | join(",")')
echo "testing features: $FEATURES"

cargo test --features $FEATURES
