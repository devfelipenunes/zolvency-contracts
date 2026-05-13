#!/bin/bash
set -e

echo "🚀 Starting Cosmos Testnet Deployment..."

# 1. Compile the contract using rust-optimizer
echo "🔨 Compiling CosmWasm contract using rust-optimizer..."
cd verifiers/cosmos

# Use docker rust-optimizer for reproducible builds (Cosmos requirement)
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.15.0

cd ../..

# 2. Run the Node.js deployment script
echo "⚙️ Executing JS deployment script..."
node scripts/deploy_cosmos_testnet.js
