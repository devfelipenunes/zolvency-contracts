#!/bin/bash
# Master script to deploy Zolvency Soul-Centric Protocol to Testnet

set -e

# Load .env
source .env

STELLAR_CLI="./.bin/stellar-cli"
NETWORK="testnet"

echo "🌟 Starting Soul-Centric Testnet Deployment..."

# 1. Deploy Zolvency Registry
echo "📦 Deploying Zolvency Registry..."
REGISTRY_WASM="target/wasm32-unknown-unknown/release/zolvency_registry.optimized.wasm"
REGISTRY_ID=$($STELLAR_CLI contract deploy \
    --wasm "$REGISTRY_WASM" \
    --source "$DEPLOYER_SECRET" \
    --network "$NETWORK")
echo "✅ Registry deployed: $REGISTRY_ID"

# 2. Deploy Zolvency Soul (Contract 1)
echo "👻 Deploying Zolvency Soul..."
SOUL_WASM="target/wasm32-unknown-unknown/release/zolvency_soul.optimized.wasm"
SOUL_ID=$($STELLAR_CLI contract deploy \
    --wasm "$SOUL_WASM" \
    --source "$DEPLOYER_SECRET" \
    --network "$NETWORK")
echo "✅ Soul deployed: $SOUL_ID"

# 3. Deploy Github Identity (Contract 3)
echo "🆔 Deploying Github Identity..."
GITHUB_WASM="target/wasm32-unknown-unknown/release/github_identity.optimized.wasm"
GITHUB_ID=$($STELLAR_CLI contract deploy \
    --wasm "$GITHUB_WASM" \
    --source "$DEPLOYER_SECRET" \
    --network "$NETWORK")
echo "✅ Github Identity deployed: $GITHUB_ID"

# 4. Deploy Uber Income (Contract 3)
# echo "🚗 Deploying Uber Income..."
# UBER_WASM="target/wasm32-unknown-unknown/release/uber_income.optimized.wasm"
# UBER_ID=$($STELLAR_CLI contract deploy \
#     --wasm "$UBER_WASM" \
#     --source "$DEPLOYER_SECRET" \
#     --network "$NETWORK")
# echo "✅ Uber Income deployed: $UBER_ID"

echo "--------------------------------"
echo "Registry:      $REGISTRY_ID"
echo "Soul (C1):     $SOUL_ID"
echo "Github (C3):   $GITHUB_ID"
echo "Uber (C3):     $UBER_ID"
echo "--------------------------------"

# Update .env EARLY
echo "Updating .env file..."
sed -i "s/^ZOLVENCY_REGISTRY_ID=.*/ZOLVENCY_REGISTRY_ID=$REGISTRY_ID/" .env || echo "ZOLVENCY_REGISTRY_ID=$REGISTRY_ID" >> .env
sed -i "s/^GITHUB_IDENTITY_ID=.*/GITHUB_IDENTITY_ID=$GITHUB_ID/" .env || echo "GITHUB_IDENTITY_ID=$GITHUB_ID" >> .env
sed -i "s/^SOUL_CONTRACT_ID=.*/SOUL_CONTRACT_ID=$SOUL_ID/" .env || echo "SOUL_CONTRACT_ID=$SOUL_ID" >> .env
sed -i "s/^UBER_INCOME_ID=.*/UBER_INCOME_ID=$UBER_ID/" .env || echo "UBER_INCOME_ID=$UBER_ID" >> .env

# ─── Initialization ───

echo "⚙️ Initializing Zolvency Registry..."
$STELLAR_CLI contract invoke --id "$REGISTRY_ID" --source "$DEPLOYER_SECRET" --network "$NETWORK" -- initialize --admin "$ADMIN_PUBLIC" --signer "$ADMIN_PUBLIC" || true

echo "⚙️ Initializing Zolvency Soul..."
$STELLAR_CLI contract invoke --id "$SOUL_ID" --source "$DEPLOYER_SECRET" --network "$NETWORK" -- initialize --admin "$ADMIN_PUBLIC" --relayer "$ADMIN_PUBLIC" || true

echo "⚙️ Initializing Github Identity..."
$STELLAR_CLI contract invoke --id "$GITHUB_ID" --source "$DEPLOYER_SECRET" --network "$NETWORK" -- initialize --admin "$ADMIN_PUBLIC" --registry "$REGISTRY_ID" --soul_contract "$SOUL_ID" --fee_token "$AXELAR_GAS_TOKEN_STELLAR" --access_control "$ADMIN_PUBLIC" --treasury "$TREASURY_ADDRESS" --mint_fee 0 || true

# echo "⚙️ Initializing Uber Income..."
# # Note: i128 values as strings to avoid parsing issues
# $STELLAR_CLI contract invoke --id "$UBER_ID" --source "$DEPLOYER_SECRET" --network "$NETWORK" -- initialize --params "{ \"admin\": \"$ADMIN_PUBLIC\", \"registry\": \"$REGISTRY_ID\", \"soul_contract\": \"$SOUL_ID\", \"fee_token\": \"$AXELAR_GAS_TOKEN_STELLAR\", \"access_control\": \"$ADMIN_PUBLIC\", \"treasury\": \"$TREASURY_ADDRESS\", \"mint_fee_30\": \"0\", \"mint_fee_60\": \"0\", \"mint_fee_90\": \"0\", \"max_proof_age_seconds\": 3600, \"store_proof_data\": true }" || true

# ─── Registry Registration ───

echo "📝 Registering tokens in Registry..."
$STELLAR_CLI contract invoke --id "$REGISTRY_ID" --source "$DEPLOYER_SECRET" --network "$NETWORK" -- register_token --admin "$ADMIN_PUBLIC" --token_contract "$GITHUB_ID" || true
# $STELLAR_CLI contract invoke --id "$REGISTRY_ID" --source "$DEPLOYER_SECRET" --network "$NETWORK" -- register_token --admin "$ADMIN_PUBLIC" --token_contract "$UBER_ID" || true

echo "🎉 Soul-Centric Deployment Process Finished!"
