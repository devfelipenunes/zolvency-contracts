#!/bin/bash
# Soul negative-path validation script (Testnet)
# Covers: double initialize, unauthorized mint, duplicate passkey.

set -e

STELLAR_CLI="./.bin/stellar-cli"
NETWORK="testnet"
SOURCE="admin"

SOURCE_PUBLIC=$($STELLAR_CLI keys address "$SOURCE")

PASSKEY="000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f00"
RECOVERY="000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f01"
PASSKEY_2="000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f02"

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
echo "   SOUL NEGATIVE PATH VALIDATION (TESTNET)        "
echo "===================================================="

$STELLAR_CLI keys generate relayer_neg --network "$NETWORK" 2>/dev/null || true
$STELLAR_CLI keys generate attacker_neg --network "$NETWORK" 2>/dev/null || true

RELAYER_ADDR=$($STELLAR_CLI keys address relayer_neg)
ATTACKER_ADDR=$($STELLAR_CLI keys address attacker_neg)

curl -s "https://friendbot.stellar.org?addr=$RELAYER_ADDR" > /dev/null
curl -s "https://friendbot.stellar.org?addr=$ATTACKER_ADDR" > /dev/null
curl -s "https://friendbot.stellar.org?addr=$SOURCE_PUBLIC" > /dev/null

echo "Deploying Soul contract..."
SOUL_ID=$($STELLAR_CLI contract deploy \
  --wasm "contracts/target/wasm32v1-none/release/zolvency_soul.wasm" \
  --source "$SOURCE" --network "$NETWORK")

echo "Initializing Soul..."
$STELLAR_CLI contract invoke --id "$SOUL_ID" --source "$SOURCE" --network "$NETWORK" --send yes -- \
  initialize --admin "$SOURCE_PUBLIC" --relayer "$RELAYER_ADDR"

expect_fail "initialize twice" \
  $STELLAR_CLI contract invoke --id "$SOUL_ID" --source "$SOURCE" --network "$NETWORK" --send yes -- \
  initialize --admin "$SOURCE_PUBLIC" --relayer "$RELAYER_ADDR"

echo "Minting first soul..."
$STELLAR_CLI contract invoke --id "$SOUL_ID" --source "relayer_neg" --network "$NETWORK" --send yes -- \
  mint --relayer "$RELAYER_ADDR" --passkey "$PASSKEY" --recovery_pubkey "$RECOVERY"

expect_fail "duplicate passkey" \
  $STELLAR_CLI contract invoke --id "$SOUL_ID" --source "relayer_neg" --network "$NETWORK" --send yes -- \
  mint --relayer "$RELAYER_ADDR" --passkey "$PASSKEY" --recovery_pubkey "$RECOVERY"

expect_fail "unauthorized relayer" \
  $STELLAR_CLI contract invoke --id "$SOUL_ID" --source "attacker_neg" --network "$NETWORK" --send yes -- \
  mint --relayer "$ATTACKER_ADDR" --passkey "$PASSKEY_2" --recovery_pubkey "$RECOVERY"

echo "===================================================="
echo "   SOUL NEGATIVE PATH VALIDATION COMPLETE         "
echo "===================================================="
