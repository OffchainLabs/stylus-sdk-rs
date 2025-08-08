#!/bin/bash

set -euo pipefail

export RUSTFLAGS="-D warnings"

# Print version information
rustc -Vv
cargo -V

# We have to use cargo metadata because we can't exclude a feature directly in cargo test.
# See: https://github.com/rust-lang/cargo/issues/3126
FEATURES=$(cargo metadata --format-version=1 --no-deps | jq -r '.packages[] | .features | keys | join("\n")')
FEATURES=$(echo "$FEATURES" | grep . | sort | uniq | grep -v default) # cleanup

# Remove integration-test because it runs in another CI job.
FEATURES=$(echo "$FEATURES" | grep -v integration-tests)

# Remove trybuild tests on nightly because they depend on the compiler output.
FEATURES=$(echo "$FEATURES" | grep -v trybuild)

FEATURES=$(echo $FEATURES | tr ' ' ',')
echo "testing features: $FEATURES"

cargo check --locked -F $FEATURES
cargo test --no-default-features -F $FEATURES

# Run trybuild tests separately without other features
if [[ "${CFG_RELEASE_CHANNEL-}" != "nightly"* ]]; then
    cargo test -p stylus-proc -F trybuild-tests
fi
