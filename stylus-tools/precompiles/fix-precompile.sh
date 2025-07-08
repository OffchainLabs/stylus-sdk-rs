#!/bin/bash

if [ -z "$1" ]; then
  echo </dev/stderr "Usage: $0 <precompile.sol>"
  exit 1
fi

sed -i \
  -e '/^interface/i #[sol(rpc)]' \
  -e '/\s*\/\*\*$/d' \
  -e 's/ \* /\/\/\/ /' \
  -e '/\s*\*\/$/d' \
  "$1"
