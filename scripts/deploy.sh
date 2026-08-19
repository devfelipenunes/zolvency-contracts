#!/bin/bash
# ZOLVENCY CORE SYSTEM DEPLOYER (soul, nexus, zpay)
# --------------------------------------------------
# Deploya apenas os contratos core mantidos neste repositório.
# Contratos auxiliares e de interop vivem em devfelipenunes/zolvency-interop.
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
echo "   ZOLVENCY CORE TESTNET DEPLOYMENT (soul/nexus/zpay)"
echo "===================================================="

# 2. DEPLOY CONTRACTS
echo "📦 Deploying core contracts to Testnet..."

NEXUS_ID=$($STELLAR_CLI contract deploy --wasm "$WASM_DIR/nexus.wasm" --source "$SOURCE" --network "$NETWORK")
echo "✅ Nexus ID: $NEXUS_ID"

SOUL_ID=$($STELLAR_CLI contract deploy --wasm "$WASM_DIR/zolvency_soul.wasm" --source "$SOURCE" --network "$NETWORK")
echo "✅ Soul ID: $SOUL_ID"

ZPAY_ID=$($STELLAR_CLI contract deploy --wasm "$WASM_DIR/zpay.wasm" --source "$SOURCE" --network "$NETWORK")
echo "✅ ZPay ID: $ZPAY_ID"

# 3. INITIALIZE CONTRACTS
echo "⚙️  Initializing system..."

echo "   -> Nexus..."
$STELLAR_CLI contract invoke --id "$NEXUS_ID" --source "$SOURCE" --network "$NETWORK" -- initialize --admin "$ADMIN_PUBLIC" --signer "$ADMIN_PUBLIC"
$STELLAR_CLI contract invoke --id "$NEXUS_ID" --source "$SOURCE" --network "$NETWORK" -- set_soul_contract --admin "$ADMIN_PUBLIC" --soul_contract "$SOUL_ID"

echo "   -> Soul..."
$STELLAR_CLI contract invoke --id "$SOUL_ID" --source "$SOURCE" --network "$NETWORK" -- initialize --admin "$ADMIN_PUBLIC" --relayer "GDZGG5MC5KQY4SPRHBENV4UEFDWGYH6IECEFUNPVWRK7Z7ZIDXRYBS5P"

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

# 4. PERSIST DATA & SYNC
echo "💾 Synchronizing registry..."

# Ensure sync script is executable
chmod +x scripts/sync_registry.sh

# Call the sync script with the new IDs
export NEXUS_ID SOUL_ID ZPAY_ID
./scripts/sync_registry.sh

echo "===================================================="
echo "   DEPLOYMENT SUCCESSFUL! SYSTEM LIVE ON TESTNET     "
echo "===================================================="
echo "Registry: contracts/registry.json"
echo "Nexus: $NEXUS_ID"
echo "Soul: $SOUL_ID"
echo "ZPay: $ZPAY_ID"
