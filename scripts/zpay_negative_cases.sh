#!/bin/bash
# ZPay negative-path validation script (Testnet)
# Covers: auth guards, token allowlist, ticket validation, oracle missing, and nexus rejection.

set -e

STELLAR_CLI="./.bin/stellar-cli"
NETWORK="testnet"
SOURCE="admin"

SOURCE_PUBLIC=$($STELLAR_CLI keys address "$SOURCE")
TREASURY_ADDRESS=$SOURCE_PUBLIC

expect_fail() {
  desc="$1"
  shift
  set +e
  "$@"
  status=$?
  set -e
  if [ $status -eq 0 ]; then
    echo "FAIL (unexpected success): $desc"
    exit 1
  fi
  echo "OK (expected failure): $desc"
}

echo "===================================================="
echo "   ZPAY NEGATIVE PATH VALIDATION (TESTNET)         "
echo "===================================================="

echo "Generating identities..."
$STELLAR_CLI keys generate user_neg --network "$NETWORK" 2>/dev/null || true
$STELLAR_CLI keys generate agent_neg --network "$NETWORK" 2>/dev/null || true
$STELLAR_CLI keys generate vendor_neg --network "$NETWORK" 2>/dev/null || true

USER_ADDR=$($STELLAR_CLI keys address user_neg)
AGENT_ADDR=$($STELLAR_CLI keys address agent_neg)
VENDOR_ADDR=$($STELLAR_CLI keys address vendor_neg)

echo "Funding accounts via Friendbot..."
curl -s "https://friendbot.stellar.org?addr=$USER_ADDR" > /dev/null
curl -s "https://friendbot.stellar.org?addr=$AGENT_ADDR" > /dev/null
curl -s "https://friendbot.stellar.org?addr=$SOURCE_PUBLIC" > /dev/null

echo "Deploying contracts..."
SOUL_ID=$($STELLAR_CLI contract deploy \
  --wasm "contracts/target/wasm32v1-none/release/zolvency_soul.wasm" \
  --source "$SOURCE" --network "$NETWORK")

NEXUS_ID=$($STELLAR_CLI contract deploy \
  --wasm "contracts/target/wasm32v1-none/release/nexus.wasm" \
  --source "$SOURCE" --network "$NETWORK")

ZPAY_ID=$($STELLAR_CLI contract deploy \
  --wasm "contracts/target/wasm32v1-none/release/zpay.wasm" \
  --source "$SOURCE" --network "$NETWORK")

TOKEN_ID="CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"

PASSKEY="000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f00"
RECOVERY="000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f01"

$STELLAR_CLI contract invoke --id "$SOUL_ID" --source "$SOURCE" --network "$NETWORK" --send yes -- \
  initialize --admin "$SOURCE_PUBLIC" --relayer "$SOURCE_PUBLIC"

$STELLAR_CLI contract invoke --id "$SOUL_ID" --source "$SOURCE" --network "$NETWORK" --send yes -- \
  mint --relayer "$SOURCE_PUBLIC" --passkey "$PASSKEY" --recovery_pubkey "$RECOVERY"

$STELLAR_CLI contract invoke --id "$NEXUS_ID" --source "$SOURCE" --network "$NETWORK" --send yes -- \
  initialize --admin "$SOURCE_PUBLIC" --signer "$SOURCE_PUBLIC"

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

$STELLAR_CLI contract invoke --id "$ZPAY_ID" --source "$SOURCE" --network "$NETWORK" --send yes -- \
  add_token --admin "$SOURCE_PUBLIC" --token "$TOKEN_ID"

MANDATE_OK=$($STELLAR_CLI contract invoke --id "$NEXUS_ID" --source "user_neg" --network "$NETWORK" --send yes -- \
  issue_mandate \
  --issuer "$USER_ADDR" \
  --agent "$AGENT_ADDR" \
  --scope "{ \"ttl\": 2000000000, \"transfer_limit\": \"1000000000\", \"scope_commitment\": null, \"contract_allowlist\": [\"$ZPAY_ID\"], \"function_allowlist\": [\"pay\"] }" \
  --delegation_policy "None" \
  --parent_mandate_id null)

MANDATE_DENY=$($STELLAR_CLI contract invoke --id "$NEXUS_ID" --source "user_neg" --network "$NETWORK" --send yes -- \
  issue_mandate \
  --issuer "$USER_ADDR" \
  --agent "$AGENT_ADDR" \
  --scope "{ \"ttl\": 2000000000, \"transfer_limit\": \"1000000000\", \"scope_commitment\": null, \"contract_allowlist\": [\"$SOUL_ID\"], \"function_allowlist\": [\"pay\"] }" \
  --delegation_policy "None" \
  --parent_mandate_id null)

NOW_TS=$(date +%s)
EXPIRED_TS=$((NOW_TS - 3600))

expect_fail "add_token with non-admin" \
  $STELLAR_CLI contract invoke --id "$ZPAY_ID" --source "user_neg" --network "$NETWORK" --send yes -- \
  add_token --admin "$USER_ADDR" --token "$TOKEN_ID"

expect_fail "initialize twice" \
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

expect_fail "pay with token not allowlisted" \
  $STELLAR_CLI contract invoke --id "$ZPAY_ID" --source "agent_neg" --network "$NETWORK" --send yes -- \
  pay \
  --agent "$AGENT_ADDR" \
  --root_anchor "$USER_ADDR" \
  --seller "$VENDOR_ADDR" \
  --token "$SOUL_ID" \
  --base_amount 100000000 \
  --mandate_id "$MANDATE_OK" \
  --price_ticket "{ \"base_currency\": \"USD\", \"price_per_unit\": \"10000000\", \"signature\": \"00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\", \"timestamp\": $NOW_TS }" \
  --oracle_feed_id null

expect_fail "price ticket expired" \
  $STELLAR_CLI contract invoke --id "$ZPAY_ID" --source "agent_neg" --network "$NETWORK" --send yes -- \
  pay \
  --agent "$AGENT_ADDR" \
  --root_anchor "$USER_ADDR" \
  --seller "$VENDOR_ADDR" \
  --token "$TOKEN_ID" \
  --base_amount 100000000 \
  --mandate_id "$MANDATE_OK" \
  --price_ticket "{ \"base_currency\": \"USD\", \"price_per_unit\": \"10000000\", \"signature\": \"00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\", \"timestamp\": $EXPIRED_TS }" \
  --oracle_feed_id null

expect_fail "invalid currency" \
  $STELLAR_CLI contract invoke --id "$ZPAY_ID" --source "agent_neg" --network "$NETWORK" --send yes -- \
  pay \
  --agent "$AGENT_ADDR" \
  --root_anchor "$USER_ADDR" \
  --seller "$VENDOR_ADDR" \
  --token "$TOKEN_ID" \
  --base_amount 100000000 \
  --mandate_id "$MANDATE_OK" \
  --price_ticket "{ \"base_currency\": \"EUR\", \"price_per_unit\": \"10000000\", \"signature\": \"00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\", \"timestamp\": $NOW_TS }" \
  --oracle_feed_id null

expect_fail "missing oracle data" \
  $STELLAR_CLI contract invoke --id "$ZPAY_ID" --source "agent_neg" --network "$NETWORK" --send yes -- \
  pay \
  --agent "$AGENT_ADDR" \
  --root_anchor "$USER_ADDR" \
  --seller "$VENDOR_ADDR" \
  --token "$TOKEN_ID" \
  --base_amount 100000000 \
  --mandate_id "$MANDATE_OK" \
  --price_ticket null \
  --oracle_feed_id null

expect_fail "nexus rejects (contract not allowlisted)" \
  $STELLAR_CLI contract invoke --id "$ZPAY_ID" --source "agent_neg" --network "$NETWORK" --send yes -- \
  pay \
  --agent "$AGENT_ADDR" \
  --root_anchor "$USER_ADDR" \
  --seller "$VENDOR_ADDR" \
  --token "$TOKEN_ID" \
  --base_amount 100000000 \
  --mandate_id "$MANDATE_DENY" \
  --price_ticket "{ \"base_currency\": \"USD\", \"price_per_unit\": \"10000000\", \"signature\": \"00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\", \"timestamp\": $NOW_TS }" \
  --oracle_feed_id null

echo "Testing Global Pause..."
$STELLAR_CLI contract invoke --id "$ZPAY_ID" --source "$SOURCE" --network "$NETWORK" --send yes -- \
  set_paused --admin "$SOURCE_PUBLIC" --paused true

expect_fail "pay when paused" \
  $STELLAR_CLI contract invoke --id "$ZPAY_ID" --source "agent_neg" --network "$NETWORK" --send yes -- \
  pay \
  --agent "$AGENT_ADDR" \
  --root_anchor "$USER_ADDR" \
  --seller "$VENDOR_ADDR" \
  --token "$TOKEN_ID" \
  --base_amount 100000000 \
  --mandate_id "$MANDATE_OK" \
  --price_ticket "{ \"base_currency\": \"USD\", \"price_per_unit\": \"10000000\", \"signature\": \"00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\", \"timestamp\": $NOW_TS }" \
  --oracle_feed_id null

echo "===================================================="
echo "   NEGATIVE PATH VALIDATION COMPLETE               "
echo "===================================================="
