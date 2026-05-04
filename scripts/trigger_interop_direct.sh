#!/bin/bash
set -e
source .env

STELLAR_CLI="./.bin/stellar-cli"

echo "1. Deploying Contracts..."
SOUL_ID=$($STELLAR_CLI contract deploy --wasm target/wasm32-unknown-unknown/release/zolvency_soul.optimized.wasm --source "$DEPLOYER_SECRET" --network testnet)
ADAPTER_ID=$($STELLAR_CLI contract deploy --wasm target/wasm32-unknown-unknown/release/zolvency_axelar_adapter.optimized.wasm --source "$DEPLOYER_SECRET" --network testnet)

echo "Soul: $SOUL_ID"
echo "Adapter: $ADAPTER_ID"

echo "2. Initializing..."
$STELLAR_CLI contract invoke --id "$SOUL_ID" --source "$DEPLOYER_SECRET" --network testnet -- initialize --admin "$ADMIN_PUBLIC" --relayer "$ADMIN_PUBLIC"
$STELLAR_CLI contract invoke --id "$ADAPTER_ID" --source "$DEPLOYER_SECRET" --network testnet -- initialize --admin "$ADMIN_PUBLIC" --soul_contract "$SOUL_ID" --gateway "$AXELAR_GATEWAY_STELLAR" --gas_service "$AXELAR_GAS_SERVICE_STELLAR" --gas_token "$AXELAR_GAS_TOKEN_STELLAR"

echo "3. Triggering Direct Cross-Chain to EVM..."
# Function send_reputation signature (9 args): 
# caller (Address), destination_chain (String), destination_address (String), external_id (String), tier (u32), user_evm_address (Bytes), nonce (u64), token_type (Symbol), ecosystem (Ecosystem)
# Note: No soul_id in this WASM version!

$STELLAR_CLI contract invoke --id "$ADAPTER_ID" --source "$DEPLOYER_SECRET" --network testnet -- send_reputation \
  --caller "$ADMIN_PUBLIC" \
  --destination_chain "ethereum-sepolia" \
  --destination_address "0x71e067692691c3A1c53D4Ab126BbEA76162BFD06" \
  --external_id "direct_evm" \
  --tier 1 \
  --user_evm_address "00" \
  --nonce 0 \
  --token_type "github" \
  --ecosystem "Evm"

echo "3. Triggering Direct Cross-Chain to Solana..."
$STELLAR_CLI contract invoke --id "$ADAPTER_ID" --source "$DEPLOYER_SECRET" --network testnet -- send_reputation \
  --caller "$ADMIN_PUBLIC" \
  --destination_chain "solana" \
  --destination_address "FM344TprtFfP39Q4Td4ZXpamaCLfhDc4Qa61ygpGcou8" \
  --external_id "direct_sol" \
  --tier 2 \
  --user_evm_address "00" \
  --nonce 1 \
  --token_type "github" \
  --ecosystem "Solana"

echo "3. Triggering Direct Cross-Chain to Cosmos..."
$STELLAR_CLI contract invoke --id "$ADAPTER_ID" --source "$DEPLOYER_SECRET" --network testnet -- send_reputation \
  --caller "$ADMIN_PUBLIC" \
  --destination_chain "osmosis-7" \
  --destination_address "osmo1mc0ysru29sepw23xazgtskrqfeu4s66q57h87w" \
  --external_id "direct_cos" \
  --tier 3 \
  --user_evm_address "00" \
  --nonce 2 \
  --token_type "github" \
  --ecosystem "Cosmos"

echo "✅ Direct Interoperability Triggered Successfully!"
