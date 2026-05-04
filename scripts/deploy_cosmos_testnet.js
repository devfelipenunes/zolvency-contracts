const { DirectSecp256k1HdWallet } = require("@cosmjs/proto-signing");
const { SigningCosmWasmClient } = require("@cosmjs/cosmwasm-stargate");
const fs = require("fs");
const path = require("path");
require("dotenv").config();

async function main() {
    const mnemonic = process.env.COSMOS_MNEMONIC;
    if (!mnemonic) {
        throw new Error("COSMOS_MNEMONIC is not set in .env");
    }

    const rpcEndpoint = process.env.COSMOS_RPC_ENDPOINT || "https://rpc.osmotest5.osmosis.zone";
    
    console.log("🌟 Connecting to Cosmos Testnet...");
    const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, { prefix: "osmo" });
    const [account] = await wallet.getAccounts();
    console.log(`Address: ${account.address}`);

    const client = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, wallet);
    const balance = await client.getBalance(account.address, "uosmo");
    console.log(`Balance: ${balance.amount} ${balance.denom}`);

    if (balance.amount === "0") {
        console.warn("⚠️ Warning: Your wallet has 0 balance. Please use the faucet for osmo-test-5.");
        return;
    }

    const wasmPath = path.join(__dirname, "..", "verifiers", "cosmos", "artifacts", "zolvency_verifier_cosmos.wasm");
    if (!fs.existsSync(wasmPath)) {
        console.error("❌ WASM file not found! Please compile the Cosmos contract first using rust-optimizer.");
        console.log("Run: docker run --rm -v \"$(pwd)/verifiers/cosmos\":/code ... cosmwasm/rust-optimizer...");
        return;
    }

    const wasmCode = fs.readFileSync(wasmPath);
    console.log("📦 Uploading WASM...");
    const uploadReceipt = await client.upload(account.address, wasmCode, "auto");
    console.log(`Code ID: ${uploadReceipt.codeId}`);

    console.log("⚙️ Instantiating contract...");
    const instantiateMsg = { admin: account.address };
    const instantiateReceipt = await client.instantiate(
        account.address,
        uploadReceipt.codeId,
        instantiateMsg,
        "Zolvency Verifier",
        "auto"
    );

    console.log("✅ Cosmos Verifier Deployed!");
    console.log(`Contract Address: ${instantiateReceipt.contractAddress}`);
}

main().catch(console.error);
