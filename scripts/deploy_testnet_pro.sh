#!/bin/bash
# ZOLVENCY PRO DEPLOYER & TESTNET VALIDATOR
# -----------------------------------------
set -e

# 1. LOAD CONFIGURATION
if [ -f .env ]; then
  echo "📄 Loading .env configuration..."
  # Use a more robust way to load .env
  set -a
  source .env
  set +a
else
  echo "❌ .env file not found! Please create one based on .env.example"
  exit 1
fi

STELLAR_CLI="./.bin/stellar-cli"
NETWORK="testnet"
# Use secret key directly as source to avoid identity mismatch
SOURCE="$ADMIN_SECRET"
WASM_DIR="target/wasm32v1-none/release"

echo "===================================================="
echo "   ZOLVENCY PROFESSIONAL TESTNET DEPLOYMENT         "
echo "===================================================="

# 2. DEPLOY CONTRACTS
echo "📦 Deploying contracts to Testnet..."

SOUL_ID=$($STELLAR_CLI contract deploy --wasm "$WASM_DIR/zolvency_soul.wasm" --source "$SOURCE" --network "$NETWORK")
echo "✅ Soul Token: $SOUL_ID"

NEXUS_ID=$($STELLAR_CLI contract deploy --wasm "$WASM_DIR/nexus.wasm" --source "$SOURCE" --network "$NETWORK")
echo "✅ Nexus: $NEXUS_ID"

ZPAY_ID=$($STELLAR_CLI contract deploy --wasm "$WASM_DIR/zpay.wasm" --source "$SOURCE" --network "$NETWORK")
echo "✅ Z-Pay: $ZPAY_ID"

# 4. INITIALIZE
echo "⚙️  Initializing stack..."

echo "   -> Soul..."
$STELLAR_CLI contract invoke --id "$SOUL_ID" --source "$SOURCE" --network "$NETWORK" --send yes -- initialize --admin "$ADMIN_PUBLIC" --relayer "$ADMIN_PUBLIC"

echo "   -> Nexus..."
$STELLAR_CLI contract invoke --id "$NEXUS_ID" --source "$SOURCE" --network "$NETWORK" --send yes -- initialize --admin "$ADMIN_PUBLIC" --signer "$ADMIN_PUBLIC"

echo "   -> Z-Pay (with Escrow logic)..."
$STELLAR_CLI contract invoke --id "$ZPAY_ID" --source "$SOURCE" --network "$NETWORK" --send yes -- \
    initialize \
    --admin "$ADMIN_PUBLIC" \
    --nexus_contract "$NEXUS_ID" \
    --oracle_pub_key 0000000000000000000000000000000000000000000000000000000000000000 \
    --service_fee 1000000 \
    --nexus_fee 500000 \
    --zpay_treasury "$TREASURY_ADDRESS" \
    --nexus_treasury "$TREASURY_ADDRESS" \
    --stork_oracle "$ADMIN_PUBLIC"

# Add Native XLM to allowlist
TOKEN_ID="CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"
echo "   -> Allowlisting XLM Native ($TOKEN_ID)..."
$STELLAR_CLI contract invoke --id "$ZPAY_ID" --source "$SOURCE" --network "$NETWORK" --send yes -- add_token --admin "$ADMIN_PUBLIC" --token "$TOKEN_ID"

# 5. PERSIST DATA
echo "💾 Saving results to .env and frontend..."

# Helper to update .env
update_env() {
    local key=$1
    local value=$2
    local file=$3
    if grep -q "^$key=" "$file"; then
        sed -i "s|^$key=.*|$key=$value|" "$file"
    else
        echo "$key=$value" >> "$file"
    fi
}

update_env "SOUL_ID" "$SOUL_ID" ".env"
update_env "NEXUS_ID" "$NEXUS_ID" ".env"
update_env "ZPAY_ID" "$ZPAY_ID" ".env"

FRONTEND_ENV="../frontend/.env.local"
if [ -f "$FRONTEND_ENV" ]; then
    echo "   -> Updating frontend/.env.local"
    update_env "NEXT_PUBLIC_SOUL_CONTRACT" "$SOUL_ID" "$FRONTEND_ENV"
    update_env "NEXT_PUBLIC_NEXUS_CONTRACT" "$NEXUS_ID" "$FRONTEND_ENV"
    update_env "NEXT_PUBLIC_ZPAY_CONTRACT" "$ZPAY_ID" "$FRONTEND_ENV"
fi

echo "===================================================="
echo "   DEPLOYMENT SUCCESSFUL! SYSTEM READY FOR AGENTS    "
echo "===================================================="
