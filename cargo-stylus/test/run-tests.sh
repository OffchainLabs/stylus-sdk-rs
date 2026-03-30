#!/bin/bash
# Run cargo-stylus tests

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Configuration
RPC_URL="${RPC_URL:-http://localhost:8547}"
CHAIN_ID="${CHAIN_ID:-412346}"
PRIVATE_KEY="${PRIVATE_KEY:-0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659}"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}=== Cargo Stylus Test Suite ===${NC}"

# Check if deployment.env exists
if [ ! -f "${SCRIPT_DIR}/deployment.env" ]; then
    echo -e "${YELLOW}No deployment found. Running deploy.sh first...${NC}"
    bash "${SCRIPT_DIR}/deploy.sh"
fi

# Load deployment info
source "${SCRIPT_DIR}/deployment.env"

if [ -z "$COUNTER_ADDRESS" ] || [ -z "$COUNTER_TX" ]; then
    echo -e "${RED}Error: Missing deployment info${NC}"
    exit 1
fi

echo "Using deployed contract: ${COUNTER_ADDRESS}"
echo "Using transaction: ${COUNTER_TX}"
echo ""

# Find cargo-stylus-beta or cargo-stylus
if command -v cargo-stylus-beta &> /dev/null; then
    CARGO_STYLUS="cargo-stylus-beta"
elif command -v cargo-stylus &> /dev/null; then
    CARGO_STYLUS="cargo-stylus"
elif [ -f "${PROJECT_DIR}/target/release/cargo-stylus-beta" ]; then
    CARGO_STYLUS="${PROJECT_DIR}/target/release/cargo-stylus-beta"
elif [ -f "${PROJECT_DIR}/target/release/cargo-stylus" ]; then
    CARGO_STYLUS="${PROJECT_DIR}/target/release/cargo-stylus"
else
    echo -e "${YELLOW}Building cargo-stylus-beta...${NC}"
    cd "${PROJECT_DIR}"
    cargo build --release
    CARGO_STYLUS="${PROJECT_DIR}/target/release/cargo-stylus-beta"
fi

echo "Using: ${CARGO_STYLUS}"

# Create lit config
cat > "${SCRIPT_DIR}/lit.site.cfg.py" << EOF
import sys
import os

config.cargo_stylus_dir = "${PROJECT_DIR}"
config.cargo_stylus = "${CARGO_STYLUS}"
config.rpc_url = "${RPC_URL}"
config.chain_id = "${CHAIN_ID}"
config.private_key = "${PRIVATE_KEY}"
config.test_contracts = {
    "counter_address": "${COUNTER_ADDRESS}",
    "counter_tx": "${COUNTER_TX}",
    # Placeholders for tests that require them
    "caller_address": "0x0000000000000000000000000000000000000000",
    "caller_tx": "0x0000000000000000000000000000000000000000000000000000000000000000",
    "complex_address": "0x0000000000000000000000000000000000000000",
    "complex_tx": "0x0000000000000000000000000000000000000000000000000000000000000000"
}

# Load the main config
lit_config.load_config(config, "${SCRIPT_DIR}/lit.cfg.py")
EOF

# Check for lit
if ! command -v lit &> /dev/null; then
    # Try llvm-lit
    LLVM_LIT=""
    for path in "/usr/local/opt/llvm/bin/llvm-lit" "/opt/homebrew/opt/llvm/bin/llvm-lit"; do
        if [ -f "$path" ]; then
            LLVM_LIT="$path"
            break
        fi
    done
    
    if [ -z "$LLVM_LIT" ]; then
        echo -e "${RED}Error: Neither 'lit' nor 'llvm-lit' found${NC}"
        echo "Install with: pip install lit"
        exit 1
    fi
    
    LIT_CMD="$LLVM_LIT"
else
    LIT_CMD="lit"
fi

# Run tests
echo -e "${YELLOW}Running tests...${NC}"
"$LIT_CMD" -v "${SCRIPT_DIR}"

echo -e "${GREEN}Test suite completed!${NC}"