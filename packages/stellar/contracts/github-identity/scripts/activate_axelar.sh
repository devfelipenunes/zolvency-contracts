#!/bin/bash

# Activate Axelar protocol in the modular Identity contract

set -e

CONTRACT_ID=$1
ADAPTER_ID=$2

if [ -z "$CONTRACT_ID" ] || [ -z "$ADAPTER_ID" ]; then
  echo "Usage: ./activate_axelar.sh <IDENTITY_CONTRACT_ID> <AXELAR_ADAPTER_CONTRACT_ID>"
  exit 1
fi

STELLAR_CLI="../../stellar-cli"

# Get the deployer address to use as admin
ADMIN=$($STELLAR_CLI keys address deployer)

echo "⚙️ Activating Axelar protocol on $CONTRACT_ID using adapter $ADAPTER_ID..."

# InteropProtocol enum: None=0, Axelar=1, LayerZero=2
# We use Axelar which is 1
PROTOCOL="Axelar"

$STELLAR_CLI contract invoke \
  --id "$CONTRACT_ID" \
  --source deployer \
  --network testnet \
  -- \
  set_active_protocol \
  --admin "$ADMIN" \
  --protocol "$PROTOCOL" \
  --adapter "$ADAPTER_ID"

echo "✅ Axelar protocol activated successfully!"
