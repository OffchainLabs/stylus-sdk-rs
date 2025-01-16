#!/bin/bash

set -euo pipefail

export RUSTFLAGS="-D warnings"
export RUSTFMT_CI=1

# Print version information
rustc -Vv
cargo -V

# Build and test main crate
if [ "$CFG_RELEASE_CHANNEL" == "nightly" ]; then
    cargo build --locked --features=docs,reentrant,hostio,mini-alloc,hostio-caching,debug
else
    cargo build --locked
fi
cargo test --features=docs,reentrant,hostio,mini-alloc,hostio-caching,debug