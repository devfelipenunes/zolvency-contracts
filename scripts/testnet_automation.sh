#!/bin/bash
# Master script to deploy Zolvency to Testnet (Stellar & EVM)

set -e

# Load .env
source .env

STELLAR_CLI="stellar"
if [ -f "./.bin/stellar-cli" ]; then
    STELLAR_CLI="./.bin/stellar-cli"
fi
# Using SEPOLIA_RPC from .env if available, otherwise fallback
SEPOLIA_RPC=${SEPOLIA_RPC:-"https://ethereum-sepolia.publicnode.com"}

echo "🌟 Starting Testnet Deployment..."

# 1. Deploy Zolvency Registry
if [ -z "$ZOLVENCY_REGISTRY_ID" ]; then
    echo "📦 Deploying Zolvency Registry..."
    REGISTRY_WASM="target/wasm32-unknown-unknown/release/zolvency_registry.wasm"
    REGISTRY_ID=$($STELLAR_CLI contract deploy \
        --wasm "$REGISTRY_WASM" \
        --source "$DEPLOYER_SECRET" \
        --network testnet)
    echo "✅ Registry deployed: $REGISTRY_ID"
else
    echo "🔄 Using existing Registry: $ZOLVENCY_REGISTRY_ID"
    REGISTRY_ID=$ZOLVENCY_REGISTRY_ID
fi

# 2. Deploy Github Identity
if [ -z "$GITHUB_IDENTITY_ID" ]; then
    echo "🆔 Deploying Github Identity..."
    IDENTITY_WASM="target/wasm32-unknown-unknown/release/github_identity.wasm"
    IDENTITY_ID=$($STELLAR_CLI contract deploy \
        --wasm "$IDENTITY_WASM" \
        --source "$DEPLOYER_SECRET" \
        --network testnet)
    echo "✅ Identity deployed: $IDENTITY_ID"
else
    echo "🔄 Using existing Identity: $GITHUB_IDENTITY_ID"
    IDENTITY_ID=$GITHUB_IDENTITY_ID
fi

# 3. Deploy EVM Verifier (Axelar)
echo "⛓️ Deploying EVM Verifier (Axelar) to Sepolia..."
cd packages/evm
# We need to set STELLAR_IDENTITY_ADDRESS for the script
export STELLAR_IDENTITY_ADDRESS=$IDENTITY_ID
# Using --rpc-url from variable
FORGE_OUTPUT=$(forge script script/DeployAxelarVerifier.s.sol --rpc-url "$SEPOLIA_RPC" --broadcast -vvvv --legacy)
VERIFIER_EVM=$(echo "$FORGE_OUTPUT" | grep "Deployed to:" | awk '{print $NF}')

if [ -z "$VERIFIER_EVM" ]; then
    echo "⚠️  EVM Deployment address not found in stdout. Checking broadcast log..."
    VERIFIER_EVM=$(jq -r '.transactions[0].contractAddress' broadcast/DeployAxelarVerifier.s.sol/11155111/run-latest.json)
fi

echo "✅ EVM Verifier deployed: $VERIFIER_EVM"
cd ../..

# 4. Initialize Identity Contract
echo "⚙️ Initializing Identity Contract..."
$STELLAR_CLI contract invoke \
    --id "$IDENTITY_ID" \
    --source "$DEPLOYER_SECRET" \
    --network testnet \
    -- \
    initialize \
    --admin "$ADMIN_PUBLIC" \
    --registry "$REGISTRY_ID" \
    --fee_token "$AXELAR_GAS_TOKEN_STELLAR" \
    --access_control "$ADMIN_PUBLIC" \
    --treasury "$TREASURY_ADDRESS" \
    --mint_fee 0

# 5. Configure Axelar Adapter
echo "🌉 Configuring Axelar Adapter..."
$STELLAR_CLI contract invoke \
    --id "$IDENTITY_ID" \
    --source "$DEPLOYER_SECRET" \
    --network testnet \
    -- \
    set_axelar_config \
    --admin "$ADMIN_PUBLIC" \
    --gateway "$AXELAR_GATEWAY_STELLAR" \
    --gas_service "$AXELAR_GAS_SERVICE_STELLAR" \
    --gas_token "$AXELAR_GAS_TOKEN_STELLAR"

# 6. Activate Axelar Protocol
echo "🚀 Activating Axelar Protocol..."
$STELLAR_CLI contract invoke \
    --id "$IDENTITY_ID" \
    --source "$DEPLOYER_SECRET" \
    --network testnet \
    -- \
    set_active_protocol \
    --admin "$ADMIN_PUBLIC" \
    --protocol Axelar \
    --adapter "$IDENTITY_ID"

echo "🎉 Deployment Complete!"
echo "Stellar Identity: $IDENTITY_ID"
echo "EVM Verifier: $VERIFIER_EVM"
