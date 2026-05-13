#!/bin/bash
set -e
source .env

STELLAR_CLI="./.bin/stellar-cli"

echo "1. Deploying Contracts..."
SOUL_ID=$($STELLAR_CLI contract deploy --wasm target/wasm32-unknown-unknown/release/zolvency_soul.optimized.wasm --source "$DEPLOYER_SECRET" --network testnet)
REGISTRY_ID=$($STELLAR_CLI contract deploy --wasm target/wasm32-unknown-unknown/release/nexus.optimized.wasm --source "$DEPLOYER_SECRET" --network testnet)
GITHUB_ID=$($STELLAR_CLI contract deploy --wasm target/wasm32-unknown-unknown/release/zolvency_github.optimized.wasm --source "$DEPLOYER_SECRET" --network testnet)
ADAPTER_ID=$($STELLAR_CLI contract deploy --wasm target/wasm32-unknown-unknown/release/zolvency_axelar_adapter.optimized.wasm --source "$DEPLOYER_SECRET" --network testnet)
MOCK_GATEWAY=$($STELLAR_CLI contract deploy --wasm target/wasm32-unknown-unknown/release/mock_gateway.optimized.wasm --source "$DEPLOYER_SECRET" --network testnet)

echo "Soul: $SOUL_ID"
echo "Registry: $REGISTRY_ID"
echo "Github: $GITHUB_ID"
echo "Adapter: $ADAPTER_ID"
echo "Mock Gateway: $MOCK_GATEWAY"

echo "2. Initializing..."
$STELLAR_CLI contract invoke --id "$SOUL_ID" --source "$DEPLOYER_SECRET" --network testnet -- initialize --admin "$ADMIN_PUBLIC" --relayer "$ADMIN_PUBLIC"
$STELLAR_CLI contract invoke --id "$REGISTRY_ID" --source "$DEPLOYER_SECRET" --network testnet -- initialize --admin "$ADMIN_PUBLIC" --signer "$ADMIN_PUBLIC"
$STELLAR_CLI contract invoke --id "$ADAPTER_ID" --source "$DEPLOYER_SECRET" --network testnet -- initialize --admin "$ADMIN_PUBLIC" --soul_contract "$SOUL_ID" --gateway "$MOCK_GATEWAY" --gas_service "$MOCK_GATEWAY" --gas_token "$MOCK_GATEWAY"

$STELLAR_CLI contract invoke --id "$REGISTRY_ID" --source "$DEPLOYER_SECRET" --network testnet -- set_interop_config --admin "$ADMIN_PUBLIC" --config '{"active_protocol": "Axelar", "adapter_address": "'$ADAPTER_ID'"}'

$STELLAR_CLI contract invoke --id "$GITHUB_ID" --source "$DEPLOYER_SECRET" --network testnet -- initialize --admin "$ADMIN_PUBLIC" --registry "$REGISTRY_ID" --soul_contract "$SOUL_ID" --fee_token "$MOCK_GATEWAY" --access_control "$ADMIN_PUBLIC" --treasury "$TREASURY_ADDRESS" --mint_fee "0"
$STELLAR_CLI contract invoke --id "$REGISTRY_ID" --source "$DEPLOYER_SECRET" --network testnet -- register_token --admin "$ADMIN_PUBLIC" --token_contract "$GITHUB_ID"

echo "3. Minting Soul..."
$STELLAR_CLI contract invoke --id "$SOUL_ID" --source "$DEPLOYER_SECRET" --network testnet -- mint --relayer "$ADMIN_PUBLIC" --passkey "00" --recovery_pubkey "00"

echo "4. Triggering Cross-Chain EVM (via Mock Gateway)..."
LONG_STR="000000000000000000000000000000000000000000000000000000000000000000000000"
PROOF='{"claim_info":{"provider":"github","parameters":"'$LONG_STR'","context":"'$LONG_STR'"},"signed_claim":"bd69905e150aeeca82b021e2de9bb2117d4fdcd1801e2cbc50b51c8c65c932ef","signatures":["31393a9e304d3a5fcc2c18dcd880419a37649def81c0097ee3793447bcdfa1794bd41412b9af5e01ccfdc8881f5c5c6f4a992f0012e15a8e73c708b741375a0f"],"witness_address":"eeb5d019f129300d714d4eedfce98af0cd02e860c0c0ec0fc86c7ede37b3696d"}'

PARAMS='{"contributions":1500,"external_id":"gh_mock","nonce":0,"username":"testuser","proof":'$PROOF'}'
CROSS_EVM='{"destination_address": "0x71e067692691c3A1c53D4Ab126BbEA76162BFD06", "destination_chain": "ethereum-sepolia", "ecosystem": "Evm", "user_destination_address": "00"}'
$STELLAR_CLI contract invoke --id "$GITHUB_ID" --source "$DEPLOYER_SECRET" --network testnet -- mint --caller "$ADMIN_PUBLIC" --soul_id 1 --params "$PARAMS" --cross_chain "$CROSS_EVM"

echo "✅ Mock Interoperability Scripts Triggered Successfully!"
