#!/bin/bash
# Deploy test contracts for cargo-stylus testing

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
CONTRACTS_DIR="${SCRIPT_DIR}/contracts"

# Configuration
RPC_URL="${RPC_URL:-http://localhost:8547}"
PRIVATE_KEY="${PRIVATE_KEY:-0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659}"

echo "Deploying test contracts"
echo "RPC URL: ${RPC_URL}"
echo ""

# Check dependencies
# Try cargo-stylus-beta first, fallback to cargo-stylus
if command -v cargo-stylus-beta &> /dev/null; then
    CARGO_STYLUS="cargo stylus-beta"
elif command -v cargo-stylus &> /dev/null; then
    CARGO_STYLUS="cargo stylus"
else
    echo "Error: neither cargo-stylus-beta nor cargo-stylus found in PATH"
    echo "Install with: cargo install --path cargo-stylus"
    exit 1
fi

echo "Using: ${CARGO_STYLUS}"

if ! command -v cast &> /dev/null; then
    echo "Error: cast not found. Install Foundry."
    exit 1
fi

# Test RPC connection
echo "Testing RPC connection..."
if ! cast chain-id --rpc-url "${RPC_URL}" &>/dev/null; then
    echo "Error: Cannot connect to RPC at ${RPC_URL}"
    exit 1
fi

# Deploy Counter contract
echo "Deploying Counter contract..."
cd "${CONTRACTS_DIR}/test-counter"

DEPLOY_OUTPUT=$(${CARGO_STYLUS} deploy \
    --private-key="${PRIVATE_KEY}" \
    --endpoint="${RPC_URL}" \
    --no-verify 2>&1)

# Extract deployed address (remove ANSI color codes)
COUNTER_ADDRESS=$(echo "${DEPLOY_OUTPUT}" | grep "deployed code at address:" | sed 's/.*deployed code at address: //' | sed 's/\x1b\[[0-9;]*m//g')

if [ -z "$COUNTER_ADDRESS" ]; then
    echo "Failed to deploy Counter"
    echo "${DEPLOY_OUTPUT}"
    exit 1
fi

echo "Counter deployed to: ${COUNTER_ADDRESS}"

# Send a test transaction
echo ""
echo "Sending test transaction..."
TX_OUTPUT=$(cast send \
    --rpc-url="${RPC_URL}" \
    --private-key="${PRIVATE_KEY}" \
    "${COUNTER_ADDRESS}" \
    "increment()" 2>&1)

# Extract transaction hash
TX_HASH=$(echo "${TX_OUTPUT}" | grep "transactionHash" | awk '{print $2}')

if [ -z "$TX_HASH" ]; then
    echo "Failed to send transaction"
    echo "${TX_OUTPUT}"
    exit 1
fi

echo "Transaction hash: ${TX_HASH}"

# Export for test suite
export COUNTER_ADDRESS
export COUNTER_TX="${TX_HASH}"

# Write deployment info to file for tests
cat > "${SCRIPT_DIR}/deployment.env" << EOF
COUNTER_ADDRESS=${COUNTER_ADDRESS}
COUNTER_TX=${TX_HASH}
EOF

echo ""
echo "Deployment complete!"
echo "Contract: ${COUNTER_ADDRESS}"
echo "Transaction: ${TX_HASH}"