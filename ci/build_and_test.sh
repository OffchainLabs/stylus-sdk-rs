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

# Remove contract-client-gen feature.
# This feature is tested individually since it considerably change the structure of the output code.
FEATURES=$(echo "$FEATURES" | grep -v contract-client-gen)

FEATURES=$(echo "$FEATURES" | tr ' ' ',')

test() {
    local features="$1"
    echo "Testing with features: $features"
    local targets="$2"

    cargo check --locked -F "$features"
    cargo test --no-default-features -F "$features" -- "$targets"
}

test "$FEATURES" ""
# disables doctests when testing contract-client-gen
test contract-client-gen "--lib --bins --tests --benches"

# Run trybuild tests separately without other features
if [[ "${CFG_RELEASE_CHANNEL-}" != "nightly"* ]]; then
    cargo test -p stylus-proc -F trybuild-tests
fi
