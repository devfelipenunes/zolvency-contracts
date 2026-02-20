#!/bin/bash

# Initialize GitHub Identity contract

set -e

if [ -z "$1" ]; then
  echo "Usage: ./initialize.sh <CONTRACT_ID>"
  exit 1
fi

CONTRACT_ID=$1

echo "🔧 Initializing contract $CONTRACT_ID..."

# Get the deployer address to use as admin
ADMIN=$(stellar keys address deployer)
ACCESS_CONTROL=$ADMIN  # For now, use same address
TREASURY=$ADMIN  # For now, use same address
MINT_FEE=0  # Free minting for testnet

echo "Admin: $ADMIN"
echo "Access Control: $ACCESS_CONTROL"
echo "Treasury: $TREASURY"
echo "Mint Fee: $MINT_FEE XLM"

stellar contract invoke \
  --id "$CONTRACT_ID" \
  --source deployer \
  --network testnet \
  -- \
  initialize \
  --admin "$ADMIN" \
  --access_control "$ACCESS_CONTROL" \
  --treasury "$TREASURY" \
  --mint_fee "$MINT_FEE"

echo "✅ Contract initialized successfully!"
