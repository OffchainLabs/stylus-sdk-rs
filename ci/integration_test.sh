#!/bin/bash

set -euo pipefail

usage() {
  echo "Usage: $0 [option]"
  echo ""
  echo "Options:"
  echo "  -e, --example <name>   Run integration tests for the specified example (e.g., erc20)"
  echo "  -s, --stylus-tools     Run integration tests for stylus-tools package only"
  echo "  -a, --all              Run all integration tests"
  echo "  -c, --clean            Clean target dirs for all tests"
  echo "  -h, --help             Display this help message"
  exit 1
}

run_example_tests() {
  local example_name="$1"
  if [ -z "$example_name" ]; then
    echo "Error: Example name not provided for --example option."
    usage
  fi
  if [ ! -d "examples/$example_name" ]; then
    echo "Error: Directory examples/$example_name not found."
    exit 1
  fi
  echo "Running integration tests for example: $example_name"
  pushd "examples/$example_name"
  cargo fmt --check
  cargo check -F integration-tests --locked --all-targets
  check_export_abi
  cargo test -F integration-tests
  popd
}

check_export_abi() {
  local abi_file="abi.sol"
  if [ ! -f "$abi_file" ]; then
    echo "Error: File '$abi_file' not found."
    exit 1
  fi
  local cargo_output=$(cargo stylus export-abi)
  local diff_output=$(diff -u <(echo "$cargo_output") "$abi_file")
  if [ -n "$diff_output" ]; then
    echo "Error: Export-abi contents do not match."
    echo "--- Diff ---"
    echo "$diff_output"
    exit 1
  fi
}

run_stylus_tools_tests() {
  echo "Running integration tests for stylus-tools package"
  cargo test -p stylus-tools -F integration-tests
}

run_all_examples() {
  echo "Running integration tests for all examples in ./examples/"
  if [ ! -d "examples" ]; then
    echo "Error: 'examples' directory not found. Skipping example tests."
    return 1
  fi
  for example_dir in examples/*/; do
    if [ -d "$example_dir" ]; then # Check if it's a directory
      local example_name
      example_name=$(basename "$example_dir")
      run_example_tests "$example_name"
    fi
  done
}

clean_all() {
  cargo clean
  for example_dir in examples/*/; do
    if [ -d "$example_dir" ]; then # Check if it's a directory
      pushd $example_dir
      cargo clean
      popd
    fi
  done
}

if [ $# -eq 0 ]; then
  usage
fi

case "$1" in
  -h|--help)
    usage
    ;;
  -e|--example)
    if [ -z "${2-}" ]; then
      echo "Error: Missing example name for $1 option."
      usage
    fi
    run_example_tests "$2"
    ;;
  -s|--stylus-tools)
    run_stylus_tools_tests
    ;;
  -a|--all)
    run_stylus_tools_tests
    run_all_examples
    ;;
  -c|--clean)
    clean_all
    ;;
  *)
    echo "Error: Unknown option $1"
    usage
    ;;
esac
