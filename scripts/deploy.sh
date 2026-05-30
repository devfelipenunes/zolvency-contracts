#!/bin/bash
# ZOLVENCY COMPLETE SYSTEM DEPLOYER
# --------------------------------
set -e

# 1. LOAD CONFIGURATION
if [ -f .env ]; then
  echo "📄 Loading .env configuration..."
  set -a
  source .env
  set +a
else
  echo "❌ .env file not found!"
  exit 1
fi

STELLAR_CLI="stellar"
NETWORK="testnet"
SOURCE="${ADMIN_SECRET:-$SOROBAN_RELAYER_SECRET}"
WASM_DIR="target/wasm32v1-none/release"

echo "===================================================="
echo "   ZOLVENCY COMPLETE TESTNET DEPLOYMENT             "
echo "===================================================="

# 2. DEPLOY CONTRACTS
echo "📦 Deploying contracts to Testnet..."

NEXUS_ID=$($STELLAR_CLI contract deploy --wasm "$WASM_DIR/nexus.wasm" --source "$SOURCE" --network "$NETWORK")
echo "✅ Nexus ID: $NEXUS_ID"

SOUL_ID=$($STELLAR_CLI contract deploy --wasm "$WASM_DIR/zolvency_soul.wasm" --source "$SOURCE" --network "$NETWORK")
echo "✅ Soul ID: $SOUL_ID"

GITHUB_ID=$($STELLAR_CLI contract deploy --wasm "$WASM_DIR/zolvency_github.wasm" --source "$SOURCE" --network "$NETWORK")
echo "✅ Github ID: $GITHUB_ID"

GIG_ID=$($STELLAR_CLI contract deploy --wasm "$WASM_DIR/zolvency_gig.wasm" --source "$SOURCE" --network "$NETWORK")
echo "✅ Gig ID: $GIG_ID"

FLOW_ID=$($STELLAR_CLI contract deploy --wasm "$WASM_DIR/zolvency_flow.wasm" --source "$SOURCE" --network "$NETWORK")
echo "✅ Flow ID: $FLOW_ID"

ADAPTER_ID=$($STELLAR_CLI contract deploy --wasm "$WASM_DIR/zolvency_axelar_adapter.wasm" --source "$SOURCE" --network "$NETWORK")
echo "✅ Axelar Adapter ID: $ADAPTER_ID"

ZPAY_ID=$($STELLAR_CLI contract deploy --wasm "$WASM_DIR/zpay.wasm" --source "$SOURCE" --network "$NETWORK")
echo "✅ ZPay ID: $ZPAY_ID"

DS_ID=$($STELLAR_CLI contract deploy --wasm "$WASM_DIR/direct_sovereign.wasm" --source "$SOURCE" --network "$NETWORK")
echo "✅ Direct Sovereign ID: $DS_ID"

# 3. INITIALIZE CONTRACTS
echo "⚙️  Initializing system..."

echo "   -> Nexus..."
$STELLAR_CLI contract invoke --id "$NEXUS_ID" --source "$SOURCE" --network "$NETWORK" -- initialize --admin "$ADMIN_PUBLIC" --signer "$ADMIN_PUBLIC"
$STELLAR_CLI contract invoke --id "$NEXUS_ID" --source "$SOURCE" --network "$NETWORK" -- set_soul_contract --admin "$ADMIN_PUBLIC" --soul_contract "$SOUL_ID"

echo "   -> Soul..."
$STELLAR_CLI contract invoke --id "$SOUL_ID" --source "$SOURCE" --network "$NETWORK" -- initialize --admin "$ADMIN_PUBLIC" --relayer "GDZGG5MC5KQY4SPRHBENV4UEFDWGYH6IECEFUNPVWRK7Z7ZIDXRYBS5P"

echo "   -> Github..."
$STELLAR_CLI contract invoke --id "$GITHUB_ID" --source "$SOURCE" --network "$NETWORK" -- initialize \
    --admin "$ADMIN_PUBLIC" \
    --registry "$NEXUS_ID" \
    --soul_contract "$SOUL_ID" \
    --fee_token "${AXELAR_GAS_TOKEN_STELLAR:-$GITHUB_ID}" \
    --access_control "$ADMIN_PUBLIC" \
    --treasury "$TREASURY_ADDRESS" \
    --mint_fee 0

echo "   -> Gig..."
# Gig uses InitializeParams struct - using strings for i128/u64 to satisfy CLI
$STELLAR_CLI contract invoke --id "$GIG_ID" --source "$SOURCE" --network "$NETWORK" -- initialize \
    --params "{ \"admin\": \"$ADMIN_PUBLIC\", \"registry\": \"$NEXUS_ID\", \"soul_contract\": \"$SOUL_ID\", \"fee_token\": \"${AXELAR_GAS_TOKEN_STELLAR:-$GITHUB_ID}\", \"access_control\": \"$ADMIN_PUBLIC\", \"treasury\": \"$TREASURY_ADDRESS\", \"mint_fee_30\": \"0\", \"mint_fee_60\": \"0\", \"mint_fee_90\": \"0\", \"max_proof_age_seconds\": 86400 }"
# Wait, let's try max_proof_age_seconds as number first, if fails I'll stringify it too.
# Actually, the error said "number, expected string or map". 
# But it pointed to the WHOLE string earlier.

echo "   -> Flow..."
$STELLAR_CLI contract invoke --id "$FLOW_ID" --source "$SOURCE" --network "$NETWORK" -- initialize --admin "$ADMIN_PUBLIC" --registry "$NEXUS_ID" --fee_token "${AXELAR_GAS_TOKEN_STELLAR:-$GITHUB_ID}" --access_control "$ADMIN_PUBLIC" --treasury "$TREASURY_ADDRESS" --mint_fee_30 0 --mint_fee_60 0 --mint_fee_90 0 --max_proof_age_seconds 86400 --is_production false
$STELLAR_CLI contract invoke --id "$FLOW_ID" --source "$SOURCE" --network "$NETWORK" -- set_soul_contract --admin "$ADMIN_PUBLIC" --soul_contract "$SOUL_ID"

echo "   -> Axelar Adapter..."
$STELLAR_CLI contract invoke --id "$ADAPTER_ID" --source "$SOURCE" --network "$NETWORK" -- initialize --admin "$ADMIN_PUBLIC" --soul_contract "$SOUL_ID" --gateway "${AXELAR_GATEWAY_STELLAR:-$GITHUB_ID}" --gas_service "${AXELAR_GAS_SERVICE_STELLAR:-$GITHUB_ID}" --gas_token "${AXELAR_GAS_TOKEN_STELLAR:-$GITHUB_ID}"

echo "   -> ZPay..."
$STELLAR_CLI contract invoke --id "$ZPAY_ID" --source "$SOURCE" --network "$NETWORK" -- initialize \
    --admin "$ADMIN_PUBLIC" \
    --nexus_contract "$NEXUS_ID" \
    --oracle_pub_key 0000000000000000000000000000000000000000000000000000000000000000 \
    --stork_oracle "$ADMIN_PUBLIC" \
    --service_fee_bps 100 \
    --min_service_fee 1000000 \
    --nexus_fee_bps 50 \
    --min_nexus_fee 500000 \
    --zpay_treasury "$TREASURY_ADDRESS" \
    --nexus_treasury "$TREASURY_ADDRESS"

echo "   -> Direct Sovereign..."
$STELLAR_CLI contract invoke --id "$DS_ID" --source "$SOURCE" --network "$NETWORK" -- initialize \
    --admin "$ADMIN_PUBLIC" \
    --nexus "$NEXUS_ID"

# 4. CROSS-LINKING & REGISTRATION
echo "🔗 Cross-linking components..."

echo "   -> Registering tokens in Nexus..."
$STELLAR_CLI contract invoke --id "$NEXUS_ID" --source "$SOURCE" --network "$NETWORK" -- register_token --admin "$ADMIN_PUBLIC" --token_contract "$GITHUB_ID"
$STELLAR_CLI contract invoke --id "$NEXUS_ID" --source "$SOURCE" --network "$NETWORK" -- register_token --admin "$ADMIN_PUBLIC" --token_contract "$GIG_ID"
$STELLAR_CLI contract invoke --id "$NEXUS_ID" --source "$SOURCE" --network "$NETWORK" -- register_token --admin "$ADMIN_PUBLIC" --token_contract "$FLOW_ID"

# 5. PERSIST DATA & SYNC
echo "💾 Synchronizing all system components..."

# Ensure sync script is executable
chmod +x scripts/sync_registry.sh

# Call the sync script with the new IDs
# Note: NEXUS_ID, SOUL_ID, etc. are already set in this script's scope
export NEXUS_ID SOUL_ID ZPAY_ID GITHUB_ID GIG_ID FLOW_ID ADAPTER_ID DS_ID
./scripts/sync_registry.sh

echo "===================================================="
echo "   DEPLOYMENT SUCCESSFUL! SYSTEM LIVE ON TESTNET     "
echo "===================================================="
echo "Registry: contracts/registry.json"
echo "Nexus: $NEXUS_ID"
echo "Soul: $SOUL_ID"
echo "ZPay: $ZPAY_ID"
