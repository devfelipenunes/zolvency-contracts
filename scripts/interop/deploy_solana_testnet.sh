#!/bin/bash
set -e

echo "🚀 Starting Solana Devnet Deployment..."

cd verifiers/solana

# Ensure dependencies are installed
if [ ! -d "node_modules" ]; then
    echo "📦 Installing Node dependencies..."
    npm install
fi

echo "🔨 Building Anchor program..."
anchor build

# The generated keypair should be set in Anchor.toml or passed as provider wallet
# Make sure your Anchor.toml is set to devnet or localnet
echo "⚙️ Deploying to Devnet..."
# Using the default keypair created earlier
export ANCHOR_WALLET=/l/disk0/fnunes/.config/solana/id.json
export ANCHOR_PROVIDER_URL=https://api.devnet.solana.com

anchor deploy --provider.cluster devnet

# Extract Program ID from target/deploy
PROGRAM_ID=$(solana address -k target/deploy/zolvency_verifier_solana-keypair.json)

echo "✅ Deployment complete!"
echo "Program ID: $PROGRAM_ID"

# Return to root
cd ../..
