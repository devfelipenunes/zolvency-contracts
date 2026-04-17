#!/bin/bash

# Initialize GitHub Identity contract

set -e

if [ -z "$1" ]; then
  echo "Usage: ./initialize.sh <CONTRACT_ID>"
  exit 1
fi

CONTRACT_ID=$1

echo "🔧 Initializing contract $CONTRACT_ID..."

STELLAR_CLI="../../stellar-cli"

# Get the deployer address to use as admin
ADMIN=$($STELLAR_CLI keys address deployer)
REGISTRY="CBO2KVJVGTQF5ZJWECXQQCXEU65ZLR5XTQZJ5MZTE6LBXEDONONJMNYF"
FEE_TOKEN="CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC" # Soroban Native Asset Contract on Testnet
ACCESS_CONTROL=$ADMIN
TREASURY=$ADMIN
MINT_FEE=0

echo "Admin: $ADMIN"
echo "Registry: $REGISTRY"
echo "Fee Token: $FEE_TOKEN"
echo "Access Control: $ACCESS_CONTROL"
echo "Treasury: $TREASURY"
echo "Mint Fee: $MINT_FEE XLM"

$STELLAR_CLI contract invoke \
  --id "$CONTRACT_ID" \
  --source deployer \
  --network testnet \
  -- \
  initialize \
  --admin "$ADMIN" \
  --registry "$REGISTRY" \
  --fee_token "$FEE_TOKEN" \
  --access_control "$ACCESS_CONTROL" \
  --treasury "$TREASURY" \
  --mint_fee $MINT_FEE

echo "✅ Contract initialized successfully!"
