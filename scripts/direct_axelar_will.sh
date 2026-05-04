#!/bin/bash
set -e
source .env

STELLAR_CLI="./.bin/stellar-cli"

echo "1. Deploying/Initializing Adapter..."
SOUL_ID=$($STELLAR_CLI contract deploy --wasm target/wasm32-unknown-unknown/release/zolvency_soul.optimized.wasm --source "$DEPLOYER_SECRET" --network testnet)
ADAPTER_ID=$($STELLAR_CLI contract deploy --wasm target/wasm32-unknown-unknown/release/zolvency_axelar_adapter.optimized.wasm --source "$DEPLOYER_SECRET" --network testnet)

$STELLAR_CLI contract invoke --id "$ADAPTER_ID" --source "$DEPLOYER_SECRET" --network testnet -- initialize --admin "$ADMIN_PUBLIC" --soul_contract "$SOUL_ID" --gateway "$AXELAR_GATEWAY_STELLAR" --gas_service "$AXELAR_GAS_SERVICE_STELLAR" --gas_token "$AXELAR_GAS_TOKEN_STELLAR"

echo "2. Triggering Direct Will Auth to EVM..."
# send_will_auth(caller, dest_chain, dest_addr, will_addr, soul_id, permissions, expiry, ecosystem)
$STELLAR_CLI contract invoke --id "$ADAPTER_ID" --source "$DEPLOYER_SECRET" --network testnet -- send_will_auth \
  --caller "$ADMIN_PUBLIC" \
  --destination_chain "ethereum-sepolia" \
  --destination_address "0x71e067692691c3A1c53D4Ab126BbEA76162BFD06" \
  --will_evm_address "00" \
  --soul_id 1 \
  --permissions 7 \
  --expiry 1735689600 \
  --ecosystem "Evm"

echo "✅ Direct Will Auth Triggered Successfully!"
