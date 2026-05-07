#!/bin/bash
# Nexus authority validation script (Testnet)
# Covers: allowlist checks and transfer limit enforcement.

set -e

STELLAR_CLI="./.bin/stellar-cli"
NETWORK="testnet"
SOURCE="admin"

SOURCE_PUBLIC=$($STELLAR_CLI keys address "$SOURCE")

echo "===================================================="
echo "   NEXUS AUTHORITY VALIDATION (TESTNET)           "
echo "===================================================="

$STELLAR_CLI keys generate issuer_neg --network "$NETWORK" 2>/dev/null || true
$STELLAR_CLI keys generate agent_neg --network "$NETWORK" 2>/dev/null || true
$STELLAR_CLI keys generate allowed_contract --network "$NETWORK" 2>/dev/null || true
$STELLAR_CLI keys generate denied_contract --network "$NETWORK" 2>/dev/null || true

ISSUER_ADDR=$($STELLAR_CLI keys address issuer_neg)
AGENT_ADDR=$($STELLAR_CLI keys address agent_neg)
ALLOW_ADDR=$($STELLAR_CLI keys address allowed_contract)
DENY_ADDR=$($STELLAR_CLI keys address denied_contract)

curl -s "https://friendbot.stellar.org?addr=$ISSUER_ADDR" > /dev/null
curl -s "https://friendbot.stellar.org?addr=$AGENT_ADDR" > /dev/null
curl -s "https://friendbot.stellar.org?addr=$SOURCE_PUBLIC" > /dev/null

echo "Deploying Nexus..."
NEXUS_ID=$($STELLAR_CLI contract deploy \
  --wasm "contracts/target/wasm32v1-none/release/nexus.wasm" \
  --source "$SOURCE" --network "$NETWORK")

echo "Initializing Nexus..."
$STELLAR_CLI contract invoke --id "$NEXUS_ID" --source "$SOURCE" --network "$NETWORK" --send yes -- \
  initialize --admin "$SOURCE_PUBLIC" --signer "$SOURCE_PUBLIC"

expect_true() {
  desc="$1"
  shift
  output=$("$@")
  echo "$output"
  if ! echo "$output" | grep -q "true"; then
    echo "FAIL (expected true): $desc"
    exit 1
  fi
  echo "OK (true): $desc"
}

expect_false() {
  desc="$1"
  shift
  output=$("$@")
  echo "$output"
  if ! echo "$output" | grep -q "false"; then
    echo "FAIL (expected false): $desc"
    exit 1
  fi
  echo "OK (false): $desc"
}

echo "Issuing mandate with allowlists..."
MANDATE_ALLOWED=$($STELLAR_CLI contract invoke --id "$NEXUS_ID" --source "issuer_neg" --network "$NETWORK" --send yes -- \
  issue_mandate \
  --issuer "$ISSUER_ADDR" \
  --agent "$AGENT_ADDR" \
  --scope "{ \"ttl\": 2000000000, \"transfer_limit\": \"100\", \"scope_commitment\": null, \"contract_allowlist\": [\"$ALLOW_ADDR\"], \"function_allowlist\": [\"pay\"] }" \
  --delegation_policy "None" \
  --parent_mandate_id null)

MANDATE_DENY_CONTRACT=$($STELLAR_CLI contract invoke --id "$NEXUS_ID" --source "issuer_neg" --network "$NETWORK" --send yes -- \
  issue_mandate \
  --issuer "$ISSUER_ADDR" \
  --agent "$AGENT_ADDR" \
  --scope "{ \"ttl\": 2000000000, \"transfer_limit\": \"100\", \"scope_commitment\": null, \"contract_allowlist\": [\"$ALLOW_ADDR\"], \"function_allowlist\": [\"pay\"] }" \
  --delegation_policy "None" \
  --parent_mandate_id null)

MANDATE_DENY_FUNCTION=$($STELLAR_CLI contract invoke --id "$NEXUS_ID" --source "issuer_neg" --network "$NETWORK" --send yes -- \
  issue_mandate \
  --issuer "$ISSUER_ADDR" \
  --agent "$AGENT_ADDR" \
  --scope "{ \"ttl\": 2000000000, \"transfer_limit\": \"100\", \"scope_commitment\": null, \"contract_allowlist\": [\"$ALLOW_ADDR\"], \"function_allowlist\": [\"pay\"] }" \
  --delegation_policy "None" \
  --parent_mandate_id null)

expect_true "allowlisted contract/function" \
  $STELLAR_CLI contract invoke --id "$NEXUS_ID" --source "$SOURCE" --network "$NETWORK" --send yes -- \
  verify_authority \
  --mandate_id "$MANDATE_ALLOWED" \
  --contract "$ALLOW_ADDR" \
  --function "pay" \
  --transfer_amount 10

expect_false "contract not in allowlist" \
  $STELLAR_CLI contract invoke --id "$NEXUS_ID" --source "$SOURCE" --network "$NETWORK" --send yes -- \
  verify_authority \
  --mandate_id "$MANDATE_DENY_CONTRACT" \
  --contract "$DENY_ADDR" \
  --function "pay" \
  --transfer_amount 10

expect_false "function not in allowlist" \
  $STELLAR_CLI contract invoke --id "$NEXUS_ID" --source "$SOURCE" --network "$NETWORK" --send yes -- \
  verify_authority \
  --mandate_id "$MANDATE_DENY_FUNCTION" \
  --contract "$ALLOW_ADDR" \
  --function "refund" \
  --transfer_amount 10

echo "Issuing mandate for budget enforcement..."
BUDGET_ID=$($STELLAR_CLI contract invoke --id "$NEXUS_ID" --source "issuer_neg" --network "$NETWORK" --send yes -- \
  issue_mandate \
  --issuer "$ISSUER_ADDR" \
  --agent "$AGENT_ADDR" \
  --scope "{ \"ttl\": 2000000000, \"transfer_limit\": \"100\", \"scope_commitment\": null, \"contract_allowlist\": [\"$ALLOW_ADDR\"], \"function_allowlist\": [\"pay\"] }" \
  --delegation_policy "None" \
  --parent_mandate_id null)

expect_true "budget under limit" \
  $STELLAR_CLI contract invoke --id "$NEXUS_ID" --source "$SOURCE" --network "$NETWORK" --send yes -- \
  verify_authority \
  --mandate_id "$BUDGET_ID" \
  --contract "$ALLOW_ADDR" \
  --function "pay" \
  --transfer_amount 60

expect_false "budget exceeded" \
  $STELLAR_CLI contract invoke --id "$NEXUS_ID" --source "$SOURCE" --network "$NETWORK" --send yes -- \
  verify_authority \
  --mandate_id "$BUDGET_ID" \
  --contract "$ALLOW_ADDR" \
  --function "pay" \
  --transfer_amount 50

echo "===================================================="
echo "   NEXUS AUTHORITY VALIDATION COMPLETE            "
echo "===================================================="
