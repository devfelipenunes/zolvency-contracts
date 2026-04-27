#!/bin/bash
# Initialize Zolvency Registry

set -e

STELLAR_CLI="stellar"
if [ -f "../../../../.bin/stellar-cli" ]; then
    STELLAR_CLI="../../../../.bin/stellar-cli"
fi
CONTRACT_ID=$1

if [ -z "$CONTRACT_ID" ]; then
  echo "Usage: ./initialize.sh <CONTRACT_ID>"
  exit 1
fi

ADMIN=$($STELLAR_CLI keys address admin)

echo "🔧 Initializing Registry $CONTRACT_ID with signer $ADMIN..."

$STELLAR_CLI contract invoke \
  --id "$CONTRACT_ID" \
  --source admin \
  --network testnet \
  -- \
  initialize \
  --admin "$ADMIN" \
  --signer "$ADMIN"

echo "✅ Registry initialized!"
