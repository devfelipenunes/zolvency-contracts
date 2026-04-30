#!/bin/bash
set -e

# Load .env
source .env

STELLAR_CLI="stellar"
if [ -f "./.bin/stellar-cli" ]; then
    STELLAR_CLI="./.bin/stellar-cli"
fi

echo "🔨 Building Soroban contracts..."
$STELLAR_CLI contract build

# 1. Deploy contracts to Stellar Testnet
echo "🚀 Deploying ZolvencySoul..."
SOUL_ID=$($STELLAR_CLI contract deploy --wasm target/wasm32v1-none/release/zolvency_soul.wasm --network testnet --source deployer)
echo "Soul ID: $SOUL_ID"

echo "🚀 Deploying ZolvencyRegistry..."
REGISTRY_ID=$($STELLAR_CLI contract deploy --wasm target/wasm32v1-none/release/zolvency_registry.wasm --network testnet --source deployer)
echo "Registry ID: $REGISTRY_ID"

echo "🚀 Deploying GithubIdentity..."
GITHUB_ID=$($STELLAR_CLI contract deploy --wasm target/wasm32v1-none/release/github_identity.wasm --network testnet --source deployer)
echo "Github ID: $GITHUB_ID"

echo "🚀 Deploying AxelarAdapter..."
ADAPTER_ID=$($STELLAR_CLI contract deploy --wasm target/wasm32v1-none/release/zolvency_axelar_adapter.wasm --network testnet --source deployer)
echo "Adapter ID: $ADAPTER_ID"

# 2. Deploy Verifier to EVM Sepolia
echo "🚀 Deploying ZolvencyVerifierAxelar to Sepolia..."
# Get deployer address for Stellar
STELLAR_DEPLOYER=$($STELLAR_CLI keys address deployer)

export STELLAR_IDENTITY_ADDRESS=$REGISTRY_ID
# Run forge script
DEPLOY_OUT=$(forge script packages/evm/script/DeployAxelarVerifier.s.sol:DeployAxelarVerifier --rpc-url $SEPOLIA_RPC --broadcast --verify -vvvv)
VERIFIER_ADDRESS=$(echo "$DEPLOY_OUT" | grep "ZolvencyVerifierAxelar deployed to:" | awk '{print $NF}')
echo "Verifier Address: $VERIFIER_ADDRESS"

# 3. Configure Stellar
echo "⚙️ Initializing ZolvencySoul..."
$STELLAR_CLI contract invoke --id "$SOUL_ID" --source deployer --network testnet -- \
  initialize --admin "$STELLAR_DEPLOYER" --relayer "$STELLAR_DEPLOYER"

echo "⚙️ Initializing ZolvencyRegistry..."
$STELLAR_CLI contract invoke --id "$REGISTRY_ID" --source deployer --network testnet -- \
  initialize --admin "$STELLAR_DEPLOYER" --signer "$STELLAR_DEPLOYER"

echo "⚙️ Initializing AxelarAdapter..."
$STELLAR_CLI contract invoke --id "$ADAPTER_ID" --source deployer --network testnet -- \
  initialize --admin "$STELLAR_DEPLOYER" --soul_contract "$SOUL_ID" \
  --gateway "$AXELAR_GATEWAY_STELLAR" --gas_service "$AXELAR_GAS_SERVICE_STELLAR" --gas_token "$AXELAR_GAS_TOKEN_STELLAR"

echo "⚙️ Configuring Interop in Registry..."
$STELLAR_CLI contract invoke --id "$REGISTRY_ID" --source deployer --network testnet -- \
  set_interop_config --admin "$STELLAR_DEPLOYER" \
  --config '{"active_protocol": "Axelar", "adapter_address": "'$ADAPTER_ID'"}'

echo "⚙️ Initializing GithubIdentity..."
$STELLAR_CLI contract invoke --id "$GITHUB_ID" --source deployer --network testnet -- \
  initialize --admin "$STELLAR_DEPLOYER" --registry "$REGISTRY_ID" --soul_contract "$SOUL_ID" \
  --fee_token "$AXELAR_GAS_TOKEN_STELLAR" --access_control "$STELLAR_DEPLOYER" --treasury "$TREASURY_ADDRESS" \
  --mint_fee 0

echo "⚙️ Registering Token in Registry..."
$STELLAR_CLI contract invoke --id "$REGISTRY_ID" --source deployer --network testnet -- \
  register_token --admin "$STELLAR_DEPLOYER" --token_contract "$GITHUB_ID"

# 4. Mint Soul ID for testing
echo "🧬 Minting Soul ID..."
$STELLAR_CLI contract invoke --id "$SOUL_ID" --source deployer --network testnet -- \
  mint --relayer "$STELLAR_DEPLOYER" --passkey "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" \
  --recovery_pubkey "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"

# 5. Execute Cross-Chain Mint
echo "📡 Triggering Cross-Chain Mint (GitHub -> Axelar -> EVM)..."
# Destination EVM address (same as deployer)
EVM_ADDR="71e067692691c3A1c53D4Ab126BbEA76162BFD06" # without 0x for Bytes conversion in CLI if needed, but let's try with 0x

# Note: Axelar expects destination address as a string on the gateway call
# Using a temp variable to avoid shell quoting issues with JSON
PARAMS='{"username": "testuser", "external_id": "test_123", "contributions": 1500, "proof_data": "", "nonce": 0}'
CROSS_CHAIN='{"destination_chain": "ethereum-sepolia", "destination_address": "'$VERIFIER_ADDRESS'", "user_destination_address": "'$EVM_ADDR'"}'

$STELLAR_CLI contract invoke --id "$GITHUB_ID" --source deployer --network testnet -- \
  mint --caller "$STELLAR_DEPLOYER" --soul_id 1 --params "$PARAMS" \
  --cross_chain "$CROSS_CHAIN"

echo "✅ Cross-chain transaction submitted!"
echo "View on Axelarscan: https://testnet.axelarscan.io/gmp/search"
echo "Registry ID: $REGISTRY_ID"
echo "Github ID: $GITHUB_ID"
echo "Verifier Address: $VERIFIER_ADDRESS"
