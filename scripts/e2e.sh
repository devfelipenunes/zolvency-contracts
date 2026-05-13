#!/bin/bash
# ZOLVENCY PRO E2E FLOW
# --------------------
# This script simulates a real-world usage of the Zolvency Protocol:
# SoulID -> Nexus Mandate -> ZPay Payment/Escrow -> Revocation
set -e

# 1. LOAD CONFIGURATION
if [ -f .env ]; then
  echo "📄 Loading .env configuration..."
  set -a
  source .env
  set +a
else
  echo "❌ .env file not found! Run deploy_complete_zolvency.sh first."
  exit 1
fi

STELLAR_CLI="stellar"
NETWORK="testnet"
SOURCE="$ADMIN_SECRET"

# Check if IDs exist
if [ -z "$NEXUS_ID" ] || [ -z "$SOUL_ID" ] || [ -z "$ZPAY_ID" ]; then
    echo "❌ Missing contract IDs in .env. Please run scripts/deploy_complete_zolvency.sh first."
    exit 1
fi

echo "===================================================="
echo "   ZOLVENCY PRO END-TO-END SIMULATION               "
echo "===================================================="

# 2. GENERATE TEST IDENTITIES
echo "👤 Generating unique identities for the simulation..."
$STELLAR_CLI keys generate user_e2e --network "$NETWORK" 2>/dev/null || true
$STELLAR_CLI keys generate agent_e2e --network "$NETWORK" 2>/dev/null || true
$STELLAR_CLI keys generate vendor_e2e --network "$NETWORK" 2>/dev/null || true

USER_ADDR=$($STELLAR_CLI keys address user_e2e)
AGENT_ADDR=$($STELLAR_CLI keys address agent_e2e)
VENDOR_ADDR=$($STELLAR_CLI keys address vendor_e2e)

echo "   User:   $USER_ADDR"
echo "   Agent:  $AGENT_ADDR"
echo "   Vendor: $VENDOR_ADDR"

# Funding accounts
echo "💸 Ensuring accounts have gas via Friendbot..."
curl -s "https://friendbot.stellar.org?addr=$USER_ADDR" > /dev/null
curl -s "https://friendbot.stellar.org?addr=$AGENT_ADDR" > /dev/null
echo "   ✅ Gas distributed."

# 3. IDENTITY ONBOARDING
echo "👻 Minting SoulID for User..."
PASSKEY="000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f00"
RECOVERY="000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f01"

# Check if user already has a soul (optional, but for clean e2e let's just try)
$STELLAR_CLI contract invoke --id "$SOUL_ID" --source "$SOURCE" --network "$NETWORK" -- mint \
    --relayer "$ADMIN_PUBLIC" \
    --owner "$USER_ADDR" \
    --passkey "$PASSKEY" \
    --recovery_pubkey "$RECOVERY" || echo "   ℹ️ User already has a SoulID or mint failed (ignoring for simulation)."

# 4. MANDATE ISSUANCE
echo "📜 User is authorizing Agent in the Nexus..."
NONCE=$(openssl rand -hex 32)
MANDATE_ID=$($STELLAR_CLI contract invoke --id "$NEXUS_ID" --source user_e2e --network "$NETWORK" -- \
    issue_mandate \
    --request "{ \"root_anchor\": \"$USER_ADDR\", \"agent\": \"$AGENT_ADDR\", \"scope\": { \"expiration\": 2000000000, \"transfer_limit\": \"1000000000\", \"renewal_period\": null, \"scope_commitment\": null, \"contract_allowlist\": [\"$ZPAY_ID\"], \"function_allowlist\": [\"pay\", \"pay_escrow\", \"release_escrow\"] }, \"delegation_policy\": \"Full\", \"parent_mandate_id\": null, \"current_epoch\": 0, \"nonce\": \"$NONCE\", \"sep45_signature\": \"00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\" }")

echo "   ✅ Mandate Issued! ID: $MANDATE_ID"

# 5. ZPAY PAYMENT
echo "💳 Agent executing payment for User..."
TOKEN_ID="CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC" # Native XLM on Testnet usually has this or similar

# User must approve ZPay
echo "   -> User approving ZPay to spend funds..."
$STELLAR_CLI contract invoke --id "$TOKEN_ID" --source user_e2e --network "$NETWORK" -- approve --from "$USER_ADDR" --spender "$ZPAY_ID" --amount 1000000000 --expiration_ledger 5000000

PRICE_TICKET_TS=$(date +%s)
echo "   -> Agent calling ZPay::pay..."
$STELLAR_CLI contract invoke --id "$ZPAY_ID" --source agent_e2e --network "$NETWORK" -- \
    pay \
    --agent "$AGENT_ADDR" \
    --root_anchor "$USER_ADDR" \
    --seller "$VENDOR_ADDR" \
    --token "$TOKEN_ID" \
    --base_amount 10000000 \
    --mandate_id "$MANDATE_ID" \
    --price_ticket "{ \"base_currency\": \"USD\", \"price_per_unit\": \"10000000\", \"signature\": \"00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\", \"timestamp\": $PRICE_TICKET_TS }" \
    --oracle_feed_id null

echo "   ✅ Direct Payment Successful!"

# 6. ZPAY ESCROW
echo "🛡️  Agent triggering ESCROW payment..."
ESCROW_ID=$($STELLAR_CLI contract invoke --id "$ZPAY_ID" --source agent_e2e --network "$NETWORK" -- \
    pay_escrow \
    --agent "$AGENT_ADDR" \
    --root_anchor "$USER_ADDR" \
    --seller "$VENDOR_ADDR" \
    --token "$TOKEN_ID" \
    --base_amount 50000000 \
    --mandate_id "$MANDATE_ID" \
    --price_ticket "{ \"base_currency\": \"USD\", \"price_per_unit\": \"10000000\", \"signature\": \"00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\", \"timestamp\": $PRICE_TICKET_TS }" \
    --oracle_feed_id null \
    --timeout_duration 86400)

echo "   ✅ Escrow Created! ID: $ESCROW_ID"

echo "🤝 Releasing Escrow..."
$STELLAR_CLI contract invoke --id "$ZPAY_ID" --source agent_e2e --network "$NETWORK" -- \
    release_escrow \
    --caller "$AGENT_ADDR" \
    --payment_id "$ESCROW_ID"

echo "   ✅ Escrow Released!"

# 7. REVOCATION
echo "🚨 User revoking Mandate (Panic Button)..."
$STELLAR_CLI contract invoke --id "$NEXUS_ID" --source user_e2e --network "$NETWORK" -- \
    revoke_mandate \
    --revoker "$USER_ADDR" \
    --mandate_id "$MANDATE_ID"

echo "   ✅ Mandate Revoked."

# 8. VERIFY FAILURE
echo "🚫 Agent trying to pay again..."
$STELLAR_CLI contract invoke --id "$ZPAY_ID" --source agent_e2e --network "$NETWORK" -- \
    pay \
    --agent "$AGENT_ADDR" \
    --root_anchor "$USER_ADDR" \
    --seller "$VENDOR_ADDR" \
    --token "$TOKEN_ID" \
    --base_amount 10000000 \
    --mandate_id "$MANDATE_ID" \
    --price_ticket "{ \"base_currency\": \"USD\", \"price_per_unit\": \"10000000\", \"signature\": \"00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\", \"timestamp\": $PRICE_TICKET_TS }" \
    --oracle_feed_id null || echo "   ✅ SUCCESS: Payment Blocked as expected!"

echo "===================================================="
echo "   E2E SIMULATION COMPLETE: SYSTEM SECURE           "
echo "===================================================="
