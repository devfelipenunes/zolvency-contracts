#!/bin/bash
# E2E Test for Zolvency Soul-Centric Flow
set -e
source .env

USER_ADDR="GCQ37NIUYYRO2LC6Z6C2Q73SCZ2FDBHHMT6FNE7BG4GM47UHXIIFJ64G"
STELLAR_CLI="../.bin/stellar"
NETWORK="testnet"

SOUL_ID="CAMORDN4NSRVEJAYX6KVD32JZXOYQ67G7HID6G3HX6S3A424XY7GK2ZC"
GITHUB_ID="CDL4J6W27G57GDWJMR2CE4WXSLAMDXJIXVMJJNWY65HDX7RVS2GPNTIV"
UBER_ID="CACCZGQ5PK4QO3TFQRVNW2KJ7ZGDMX5NF2XC7N6N6KEG7A5DXN2TF6FC"

echo "🧪 Starting E2E Flow Test for $USER_ADDR..."

# 1. Mint Soul
echo "1️⃣ Minting Soul..."
$STELLAR_CLI contract invoke --id "$SOUL_ID" --source "$DEPLOYER_SECRET" --network "$NETWORK" --send yes -- mint --relayer "$ADMIN_PUBLIC" --user "$USER_ADDR" --passkey 000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f

# 2. Verify Soul
echo "✅ Verifying Soul Balance..."
BALANCE=$($STELLAR_CLI contract invoke --id "$SOUL_ID" --source "$DEPLOYER_SECRET" --network "$NETWORK" -- balance --user "$USER_ADDR")
echo "Balance: $BALANCE"
if [ "$BALANCE" != "1" ]; then echo "❌ Error: Soul balance should be 1"; exit 1; fi

# 3. Mint Github (Gated)
echo "2️⃣ Minting Github rSBT (Soul Gating Check)..."
$STELLAR_CLI contract invoke --id "$GITHUB_ID" --source "$DEPLOYER_SECRET" --network "$NETWORK" --send yes -- mint --caller "$ADMIN_PUBLIC" --user "$USER_ADDR" --params "{ \"contributions\": 500, \"external_id\": \"gh_final_test\", \"nonce\": 0, \"passkey\": \"00\", \"passkey_signature\": \"00\", \"proof_data\": \"00\", \"username\": \"gh_final_test\" }"

# 4. Mint Uber Income (Gated)
echo "3️⃣ Minting Uber Income rSBT (Soul Gating Check)..."
NOW=$(date +%s)
$STELLAR_CLI contract invoke --id "$UBER_ID" --source "$DEPLOYER_SECRET" --network "$NETWORK" --send yes -- mint --admin "$ADMIN_PUBLIC" --params "{ \"recipient\": \"$USER_ADDR\", \"external_id\": \"uber_final_test\", \"income_band\": 5, \"income_value\": null, \"reveal_mode\": \"Band\", \"currency\": \"USD\", \"period\": \"Monthly\", \"verified_at\": $NOW, \"proof_hash\": \"0000000000000000000000000000000000000000000000000000000000000000\", \"proof_data\": \"00\", \"window\": \"Days30\", \"nonce\": 0 }"

echo "🎉 E2E Flow Test Completed Successfully!"
