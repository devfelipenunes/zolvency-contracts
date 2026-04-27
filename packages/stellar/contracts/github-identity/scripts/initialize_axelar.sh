#!/bin/bash

# Initialize Axelar configuration for the GitHub Identity contract on Stellar testnet

set -e

CONTRACT_ID=$1

if [ -z "$CONTRACT_ID" ]; then
  echo "Usage: ./initialize_axelar.sh <CONTRACT_ID>"
  exit 1
fi

STELLAR_CLI="stellar"
if [ -f "../../../../.bin/stellar-cli" ]; then
    STELLAR_CLI="../../../../.bin/stellar-cli"
fi

# Addresses from .env (make sure to set them or they will use defaults)
GATEWAY=${AXELAR_GATEWAY_STELLAR:-"CCSNWHMQSPTW4PS7L32OIMH7Z6NFNCKYZKNFSWRSYX7MK64KHBDZDT5I"}
GAS_SERVICE=${AXELAR_GAS_SERVICE_STELLAR:-"CAZUKAFB5XHZKFZR7B5HIKB6BBMYSZIV3V2VWFTQWKYEMONWK2ZLTZCT"}
GAS_TOKEN=${AXELAR_GAS_TOKEN_STELLAR:-"CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"}

echo "⚙️ Configuring Axelar on $CONTRACT_ID..."

$STELLAR_CLI contract invoke \
  --id "$CONTRACT_ID" \
  --source deployer \
  --network testnet \
  -- \
  set_axelar_config \
  --admin deployer \
  --gateway "$GATEWAY" \
  --gas_service "$GAS_SERVICE" \
  --gas_token "$GAS_TOKEN"

echo "✅ Axelar configured successfully!"
