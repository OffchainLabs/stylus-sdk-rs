# Cargo Stylus Test Suite

This directory contains the LLVM lit-based test suite for the `cargo-stylus` replay and usertrace commands.

## Prerequisites

1. **LLVM lit**: Install with `pip install lit`
2. **Foundry (cast)**: Install from https://getfoundry.sh/
3. **Local Nitro Node**: Start with:
   ```bash
   docker run -it --rm --name nitro-dev -p 8547:8547 \
     offchainlabs/nitro-node:v3.5.3-rc.3-653b078 \
     --dev --http.addr 0.0.0.0 --http.api=net,web3,eth,arb,arbdebug,debug
   ```
4. **FileCheck**: Part of LLVM tools (usually available with LLVM installation)
5. **stylusdb** (optional): For full replay/usertrace testing
6. **cargo-stylus**: Install with `cargo install --path cargo-stylus` or build from this project

## Running Tests

From the cargo-stylus root directory:

```bash
# Run all tests
./test/run-tests.sh

# Run specific test category
lit -v test/replay/
lit -v test/usertrace/

# Run with custom RPC
RPC_URL=http://localhost:8545 ./test/run-tests.sh
```

## Writing New Tests

1. Create a `.test` file in the appropriate directory
2. Use FileCheck directives to verify output:
   ```
   # RUN: %cargo-stylus replay --tx-hash %{counter_tx} --rpc-url %{rpc_url} 2>&1 | FileCheck %s
   # CHECK: Starting debug session
   ```
