#!/bin/bash
# Build and deploy Zolvency Registry to Stellar testnet

set -e

STELLAR_CLI="stellar"
if [ -f "../../../../.bin/stellar-cli" ]; then
    STELLAR_CLI="../../../../.bin/stellar-cli"
fi

echo "🔨 Building Registry contract (optimized)..."
$STELLAR_CLI contract build --optimize

WASM_PATH="$(dirname "$0")/../target/wasm32v1-none/release/zolvency_registry.wasm"

if [ ! -f "$WASM_PATH" ]; then
  echo "❌ Unable to find wasm file at: $WASM_PATH"
  exit 1
fi

echo "🚀 Deploying Registry to testnet..."
CONTRACT_ID=$($STELLAR_CLI contract deploy \
  --wasm "$WASM_PATH" \
  --network testnet \
  --source deployer)

echo "✅ Registry deployed!"
echo "Contract ID: $CONTRACT_ID"
echo "Save this to your .env: REGISTRY_CONTRACT_ID=$CONTRACT_ID"
