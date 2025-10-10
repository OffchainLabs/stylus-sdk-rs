#!/bin/bash

set -euo pipefail

usage() {
  echo "Usage: $0 [option]"
  echo ""
  echo "Options:"
  echo "  -c, --contract-client-gen                 Run clippy on stylus-proc with contract-client-gen feature"
  echo "  -e, --export-abi                          Run clippy on stylus-proc with export-abi feature"
  echo "  -a, --all-excluding-special-features      Run clippy excluding contract-client-gen and export-abi features"
  echo "  -h, --help                                Display this help message"
}

run_all_clippy() {
  echo "Running clippy on all packages excluding contract-client-gen and export-abi features..."

  # Get all crates in the workspace
  workspace_members=$(cargo metadata --format-version=1 | jq -r '.workspace_members[] | split(" ")[0]')

  # For each crate, run clippy with all features except contract-client-gen
  for crate in $workspace_members; do
    if [[ "$crate" == *"erc20"* || "$crate" == *"erc721"* ]]; then
      continue
    fi
    # Get features for this crate excluding the ones we don't want
    features=$(cargo metadata --format-version=1 \
      | jq -r ".packages[] | select(.id == \"$crate\") | .features | keys[] |
              select(. != \"contract-client-gen\" and . != \"export-abi\")" \
      | tr '\n' ',' | sed 's/,$//')

    echo "Running clippy on $crate with features: $features"
    if [ -n "$features" ]; then
      cargo clippy -p "$crate" --all-targets --no-default-features --features "$features" -- -D warnings
    else
      cargo clippy -p "$crate" --all-targets --no-default-features -- -D warnings
    fi
  done
}

run_contract_client_gen_clippy() {
  echo "Running clippy on stylus-proc with contract-client-gen feature..."
  cargo clippy -p stylus-proc --no-default-features --features "contract-client-gen" --all-targets -- -D warnings
}

run_export_abi_clippy() {
  echo "Running clippy on stylus-proc with export-abi feature..."
  cargo clippy -p stylus-proc --no-default-features --features "export-abi" --all-targets -- -D warnings
}

if [ $# -eq 0 ]; then
  usage
fi

case "$1" in
  -h | --help)
    usage
    ;;
  -c | --contract-client-gen)
    run_contract_client_gen_clippy
    ;;
  -e | --export-abi)
    run_export_abi_clippy
    ;;
  -a | --all-excluding-special-features)
    run_all_clippy
    ;;
  *)
    echo "Error: Unknown option $1"
    usage
    exit 1
    ;;
esac
