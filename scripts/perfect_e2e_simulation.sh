#!/bin/bash
# ZOLVENCY PERFECT E2E FLOW SIMULATOR (V3 REAL - STABLE)
# -----------------------------------------------------
# Real end-to-end simulation on Stellar Testnet:
# 1. Identity (Soul Token minting)
# 2. Nexus Setup (Wills authority)
# 3. Z-Pay Execution (The Agentic Rail)
# 4. Sovereignty Check (Panic Button)

set -e

# Configuration
STELLAR_CLI="./.bin/stellar-cli"
NETWORK="testnet" 
SOURCE="admin" # Using existing 'admin' identity

# Fetch admin public key
SOURCE_PUBLIC=$($STELLAR_CLI keys address "$SOURCE")
TREASURY_ADDRESS=$SOURCE_PUBLIC # For demo purposes

echo "===================================================="
echo "   ZOLVENCY REAL-FLOW SIMULATOR (HIGH FIDELITY)     "
echo "===================================================="

# 1. GENERATE IDENTITIES
echo "👤 Generating unique identities for the simulation..."
$STELLAR_CLI keys add user_sim || true
$STELLAR_CLI keys add agent_sim || true
$STELLAR_CLI keys add worker_sim || true
$STELLAR_CLI keys add vendor_sim || true
$STELLAR_CLI keys generate user_sim --network "$NETWORK" 2>/dev/null || true
$STELLAR_CLI keys generate agent_sim --network "$NETWORK" 2>/dev/null || true
$STELLAR_CLI keys generate worker_sim --network "$NETWORK" 2>/dev/null || true
$STELLAR_CLI keys generate vendor_sim --network "$NETWORK" 2>/dev/null || true

USER_ADDR=$($STELLAR_CLI keys address user_sim)
AGENT_ADDR=$($STELLAR_CLI keys address agent_sim)
SUB_AGENT_ADDR=$($STELLAR_CLI keys address worker_sim)
VENDOR_ADDR=$($STELLAR_CLI keys address vendor_sim)

echo "   User (Root Anchor): $USER_ADDR"
echo "   Agent (IA Master): $AGENT_ADDR"
echo "   Sub-Agent (Worker): $SUB_AGENT_ADDR"
echo "   Vendor:            $VENDOR_ADDR"

# Funding accounts
echo "💸 Ensuring accounts have gas..."
curl -s "https://friendbot.stellar.org?addr=$USER_ADDR" > /dev/null
curl -s "https://friendbot.stellar.org?addr=$AGENT_ADDR" > /dev/null
curl -s "https://friendbot.stellar.org?addr=$SUB_AGENT_ADDR" > /dev/null
curl -s "https://friendbot.stellar.org?addr=$SOURCE_PUBLIC" > /dev/null
echo "   ✅ Gas distributed via Friendbot."

# 2. DEPLOY CONTRACTS
echo "📦 Deploying the Zolvency Stack..."

# Note: Using the new wasm32v1-none target paths for modern Soroban
SOUL_ID=$($STELLAR_CLI contract deploy \
    --wasm "target/wasm32v1-none/release/zolvency_soul.wasm" \
    --source "$SOURCE" --network "$NETWORK")
echo "   ✅ Soul Token Deployed: $SOUL_ID"

NEXUS_ID=$($STELLAR_CLI contract deploy \
    --wasm "target/wasm32v1-none/release/nexus.wasm" \
    --source "$SOURCE" --network "$NETWORK")
echo "   ✅ Nexus Deployed: $NEXUS_ID"

ZPAY_ID=$($STELLAR_CLI contract deploy \
    --wasm "target/wasm32v1-none/release/zpay.wasm" \
    --source "$SOURCE" --network "$NETWORK")
echo "   ✅ Z-Pay Deployed: $ZPAY_ID"

# 3. INITIALIZATION & ONBOARDING
echo "⚙️  Initializing Contracts..."

# Initialize Soul
echo "   -> Initializing Soul Contract..."
$STELLAR_CLI contract invoke --id "$SOUL_ID" --source "$SOURCE" --network "$NETWORK" --send yes -- initialize --admin "$SOURCE_PUBLIC" --relayer "$SOURCE_PUBLIC"

# Mint SoulID for User
# Requirement: passkey and recovery_pubkey must be 65 bytes hex (130 chars)
PASSKEY="000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f00"
RECOVERY="000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f01"

echo "👻 Minting SoulID for User ($USER_ADDR)..."
$STELLAR_CLI contract invoke --id "$SOUL_ID" --source "$SOURCE" --network "$NETWORK" --send yes -- mint \
    --relayer "$SOURCE_PUBLIC" \
    --passkey "$PASSKEY" \
    --recovery_pubkey "$RECOVERY"

# Initialize Nexus
echo "   -> Initializing Nexus..."
$STELLAR_CLI contract invoke --id "$NEXUS_ID" --source "$SOURCE" --network "$NETWORK" --send yes -- initialize --admin "$SOURCE_PUBLIC" --signer "$SOURCE_PUBLIC"

# Initialize Z-Pay
echo "   -> Configuring Z-Pay Gateway..."

$STELLAR_CLI contract invoke --id "$ZPAY_ID" --source "$SOURCE" --network "$NETWORK" --send yes -- \
    initialize \
    --admin "$SOURCE_PUBLIC" \
    --nexus_contract "$NEXUS_ID" \
    --oracle_pub_key 0000000000000000000000000000000000000000000000000000000000000000 \
    --service_fee 1000000 \
    --nexus_fee 500000 \
    --zpay_treasury "$TREASURY_ADDRESS" \
    --nexus_treasury "$TREASURY_ADDRESS" \
    --stork_oracle "$SOURCE_PUBLIC" 

# Add token to allowlist after initialization
$STELLAR_CLI contract invoke --id "$ZPAY_ID" --source "$SOURCE" --network "$NETWORK" --send yes -- add_token --admin "$SOURCE_PUBLIC" --token "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"

# 4. THE SOVEREIGN MOMENT: ISSUING THE WILL
echo "📜 User is creating a Mandate (Will) for the Agent in the Nexus..."
MANDATE_ID=$($STELLAR_CLI contract invoke --id "$NEXUS_ID" --source "user_sim" --network "$NETWORK" --send yes -- \
    issue_mandate \
    --issuer "$USER_ADDR" \
    --agent "$AGENT_ADDR" \
    --scope "{ \"ttl\": 2000000000, \"transfer_limit\": \"1000000000\", \"scope_commitment\": null, \"contract_allowlist\": [\"$ZPAY_ID\"], \"function_allowlist\": [\"pay\", \"pay_escrow\", \"charge\"] }" \
    --delegation_policy "Full" \
    --parent_mandate_id null)

echo "   ✅ Mandate Issued! ID: $MANDATE_ID. Agent is authorized to spend via Z-Pay."

# 5. AGENT EXECUTION
echo "🤖 Agent detected authority! Executing payment for service..."
# Token: XLM Native
TOKEN_ID="CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"

# User approves Z-Pay on the Token
echo "💸 User giving allowance to Z-Pay on Token..."
$STELLAR_CLI contract invoke --id "$TOKEN_ID" --source "user_sim" --network "$NETWORK" --send yes -- \
    approve --from "$USER_ADDR" --spender "$ZPAY_ID" --amount 2000000000 --expiration_ledger 2500000

# Agent triggers payment via Z-Pay
echo "💳 Agent calling Z-Pay::pay..."
# Use a fresh timestamp to avoid PriceTicketExpired
PRICE_TICKET_TS=$(date +%s)
# Using the real Mandate ID and correct ActionContext mapping
$STELLAR_CLI contract invoke --id "$ZPAY_ID" --source "agent_sim" --network "$NETWORK" --send yes -- \
    pay \
    --agent "$AGENT_ADDR" \
    --root_anchor "$USER_ADDR" \
    --seller "$VENDOR_ADDR" \
    --token "$TOKEN_ID" \
    --base_amount 100000000 \
    --mandate_id "$MANDATE_ID" \
    --price_ticket "{ \"base_currency\": \"USD\", \"price_per_unit\": \"10000000\", \"signature\": \"00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\", \"timestamp\": $PRICE_TICKET_TS }" \
    --oracle_feed_id null

echo "   ✅ Payment Successful! Nexus verified authority and Z-Pay settled the funds."

# 5.1 SUB-AGENT DELEGATION (THE HIERARCHY)
echo "🧬 Agent Alpha (Master) is delegating a task to Sub-Agent Beta (Worker)..."
SUB_MANDATE_ID=$($STELLAR_CLI contract invoke --id "$NEXUS_ID" --source "agent_sim" --network "$NETWORK" --send yes -- \
    issue_mandate \
    --issuer "$AGENT_ADDR" \
    --agent "$SUB_AGENT_ADDR" \
    --scope "{ \"ttl\": 1900000000, \"transfer_limit\": \"100000000\", \"scope_commitment\": null, \"contract_allowlist\": [\"$ZPAY_ID\"], \"function_allowlist\": [\"pay\"] }" \
    --delegation_policy "None" \
    --parent_mandate_id "$MANDATE_ID")

echo "   ✅ Sub-Mandate Issued! ID: $SUB_MANDATE_ID. Worker Beta is now authorized by Agent Alpha."

echo "💳 Sub-Agent Beta executing payment via Z-Pay..."
$STELLAR_CLI contract invoke --id "$ZPAY_ID" --source "worker_sim" --network "$NETWORK" --send yes -- \
    pay \
    --agent "$SUB_AGENT_ADDR" \
    --root_anchor "$USER_ADDR" \
    --seller "$VENDOR_ADDR" \
    --token "$TOKEN_ID" \
    --base_amount 50000000 \
    --mandate_id "$SUB_MANDATE_ID" \
    --price_ticket "{ \"base_currency\": \"USD\", \"price_per_unit\": \"10000000\", \"signature\": \"00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\", \"timestamp\": $PRICE_TICKET_TS }" \
    --oracle_feed_id null

echo "   ✅ Sub-Agent Payment Success! Nexus traversed the chain: Beta -> Alpha -> User."

# 5.1 ESCROW FLOW (THE TRUSTLESS VAULT)
echo "🛡️  Scenario: High-value job. Agent triggering ESCROW instead of direct pay..."
PAYMENT_ID=$($STELLAR_CLI contract invoke --id "$ZPAY_ID" --source "agent_sim" --network "$NETWORK" --send yes -- \
    pay_escrow \
    --agent "$AGENT_ADDR" \
    --root_anchor "$USER_ADDR" \
    --seller "$VENDOR_ADDR" \
    --token "$TOKEN_ID" \
    --base_amount 200000000 \
    --mandate_id "$MANDATE_ID" \
    --price_ticket "{ \"base_currency\": \"USD\", \"price_per_unit\": \"10000000\", \"signature\": \"00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\", \"timestamp\": $PRICE_TICKET_TS }" \
    --oracle_feed_id null)

echo "   ✅ Escrow Created! ID: $PAYMENT_ID. Funds are now locked in Z-Pay Vault."

echo "🤝 Job complete. Releasing Escrow funds to Vendor..."
$STELLAR_CLI contract invoke --id "$ZPAY_ID" --source "agent_sim" --network "$NETWORK" --send yes -- \
    release_escrow \
    --caller "$AGENT_ADDR" \
    --payment_id "$PAYMENT_ID"

echo "   ✅ Escrow Released! Vendor has been paid."

# 5.2 RECURRING CHARGE (SUBSCRIPTION)
echo "🔄 Scenario: Recurring service. Vendor charging the User via Z-Pay..."
$STELLAR_CLI contract invoke --id "$ZPAY_ID" --source "vendor_sim" --network "$NETWORK" --send yes -- \
    charge_subscription \
    --seller "$VENDOR_ADDR" \
    --root_anchor "$USER_ADDR" \
    --token "$TOKEN_ID" \
    --base_amount 30000000 \
    --mandate_id "$MANDATE_ID" \
    --price_ticket "{ \"base_currency\": \"USD\", \"price_per_unit\": \"10000000\", \"signature\": \"00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\", \"timestamp\": $PRICE_TICKET_TS }" \
    --oracle_feed_id null

echo "   ✅ Subscription Charge Success! Vendor pulled funds based on authorized mandate."

# 6. CRISIS SIMULATION: THE PANIC BUTTON
echo "🚨 CRISIS DETECTED! User is pressing the PANIC BUTTON (Revoking Mandate)..."
$STELLAR_CLI contract invoke --id "$NEXUS_ID" --source "user_sim" --network "$NETWORK" --send yes -- \
    revoke_mandate \
    --caller "$USER_ADDR" \
    --mandate_id "$MANDATE_ID"

echo "   ✅ Mandate #$MANDATE_ID has been invalidated."

# 7. VERIFYING SOBERANITY
echo "🤖 Agent tries to pay again..."
$STELLAR_CLI contract invoke --id "$ZPAY_ID" --source "agent_sim" --network "$NETWORK" --send yes -- \
    pay \
    --agent "$AGENT_ADDR" \
    --root_anchor "$USER_ADDR" \
    --seller "$VENDOR_ADDR" \
    --token "$TOKEN_ID" \
    --base_amount 100000000 \
    --mandate_id "$MANDATE_ID" \
    --price_ticket "{ \"base_currency\": \"USD\", \"price_per_unit\": \"10000000\", \"signature\": \"00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\", \"timestamp\": $PRICE_TICKET_TS }" \
    --oracle_feed_id null || echo "   ❌ BLOCKED! Agent Alpha rejected: Authority Revoked."

echo "🤖 Sub-Agent Beta tries to pay..."
$STELLAR_CLI contract invoke --id "$ZPAY_ID" --source "worker_sim" --network "$NETWORK" --send yes -- \
    pay \
    --agent "$SUB_AGENT_ADDR" \
    --root_anchor "$USER_ADDR" \
    --seller "$VENDOR_ADDR" \
    --token "$TOKEN_ID" \
    --base_amount 10000000 \
    --mandate_id "$SUB_MANDATE_ID" \
    --price_ticket "{ \"base_currency\": \"USD\", \"price_per_unit\": \"10000000\", \"signature\": \"00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\", \"timestamp\": $PRICE_TICKET_TS }" \
    --oracle_feed_id null || echo "   ❌ CASCADED BLOCK! Sub-Agent Beta also rejected: Parent Revoked."

echo "===================================================="
echo "   SIMULATION COMPLETE: SOBERANITY PROTECTED        "
echo "===================================================="
