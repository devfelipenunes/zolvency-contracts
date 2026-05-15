#!/bin/bash
# ZOLVENCY CONTRACT SYNC UTILITY
# ------------------------------
# This script ensures all .env files across the repository are in sync.

# Get the root directory
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REGISTRY_FILE="$ROOT_DIR/contracts/registry.json"

echo "🔄 Synchronizing contract addresses..."

# Function to update a variable in a .env file
update_env_file() {
    local key=$1
    local value=$2
    local file=$3
    
    if [ ! -f "$file" ]; then
        return
    fi

    if grep -q "^$key=" "$file"; then
        # Use a different delimiter for sed in case value contains /
        sed -i "s|^$key=.*|$key=$value|" "$file"
    else
        echo "$key=$value" >> "$file"
    fi
}

# List of files to sync
ENV_FILES=(
    "$ROOT_DIR/.env"
    "$ROOT_DIR/contracts/.env"
    "$ROOT_DIR/zpay/zpay-mcp/.env"
    "$ROOT_DIR/soulid/.env"
    "$ROOT_DIR/frontend/.env.local"
)

# Contract mappings (Standard Name -> Variable Names in different files)
# Structure: StandardName|Value|Alias1|Alias2...
sync_contract() {
    local std_name=$1
    local value=$2
    shift 2
    local aliases=("$@")

    for file in "${ENV_FILES[@]}"; do
        if [ -f "$file" ]; then
            # Update standard name
            update_env_file "$std_name" "$value" "$file"
            # Update aliases
            for alias in "${aliases[@]}"; do
                update_env_file "$alias" "$value" "$file"
            done
        fi
    done
}

# Load current values from registry if it exists, otherwise use what's passed
# For now, we assume values are passed as arguments or environment variables

# Expected environment variables:
# NEXUS_ID, SOUL_ID, ZPAY_ID, GITHUB_ID, GIG_ID, FLOW_ID, ADAPTER_ID, DS_ID

sync_contract "NEXUS_CONTRACT_ID" "$NEXUS_ID" "NEXUS_ID" "ZPAY_CONTRACT_ID_NEXUS" "ZOLVENCY_HUB_ADDRESS"
sync_contract "SOUL_CONTRACT_ID" "$SOUL_ID" "SOUL_ID" "ZPAY_CONTRACT_ID_IDENTITY"
sync_contract "ZPAY_CONTRACT_ID" "$ZPAY_ID" "ZPAY_ID" "ZPAY_CONTRACT_ID_GATEWAY"
sync_contract "GITHUB_CONTRACT_ID" "$GITHUB_ID" "GITHUB_ID"
sync_contract "GIG_CONTRACT_ID" "$GIG_ID" "GIG_ID"
sync_contract "FLOW_CONTRACT_ID" "$FLOW_ID" "FLOW_ID"
sync_contract "AXELAR_ADAPTER_ID" "$ADAPTER_ID" "AXELAR_ADAPTER_ID"
sync_contract "DIRECT_SOVEREIGN_ID" "$DS_ID" "DIRECT_SOVEREIGN_ID"
sync_contract "NEXT_PUBLIC_WALLET_WASM_HASH" "$WASM_HASH" "NEXT_PUBLIC_WALLET_WASM_HASH"
sync_contract "NEXT_PUBLIC_GITHUB_IDENTITY_CONTRACT" "$GITHUB_ID" "NEXT_PUBLIC_GITHUB_IDENTITY_CONTRACT"
sync_contract "SOROBAN_RELAYER_SECRET" "$RELAYER_SECRET" "SOROBAN_RELAYER_SECRET"

# Update Registry JSON
echo "{" > "$REGISTRY_FILE"
echo "  \"NEXUS_CONTRACT_ID\": \"$NEXUS_ID\"," >> "$REGISTRY_FILE"
echo "  \"SOUL_CONTRACT_ID\": \"$SOUL_ID\"," >> "$REGISTRY_FILE"
echo "  \"ZPAY_CONTRACT_ID\": \"$ZPAY_ID\"," >> "$REGISTRY_FILE"
echo "  \"GITHUB_CONTRACT_ID\": \"$GITHUB_ID\"," >> "$REGISTRY_FILE"
echo "  \"GIG_CONTRACT_ID\": \"$GIG_ID\"," >> "$REGISTRY_FILE"
echo "  \"FLOW_CONTRACT_ID\": \"$FLOW_ID\"," >> "$REGISTRY_FILE"
echo "  \"AXELAR_ADAPTER_ID\": \"$ADAPTER_ID\"," >> "$REGISTRY_FILE"
echo "  \"DIRECT_SOVEREIGN_ID\": \"$DS_ID\"," >> "$REGISTRY_FILE"
echo "  \"NEXT_PUBLIC_WALLET_WASM_HASH\": \"$WASM_HASH\"," >> "$REGISTRY_FILE"
echo "  \"NEXT_PUBLIC_GITHUB_IDENTITY_CONTRACT\": \"$GITHUB_ID\"," >> "$REGISTRY_FILE"
echo "  \"SOROBAN_RELAYER_SECRET\": \"$RELAYER_SECRET\"" >> "$REGISTRY_FILE"
echo "}" >> "$REGISTRY_FILE"

echo "✅ Synchronization complete. Registry updated at $REGISTRY_FILE"
