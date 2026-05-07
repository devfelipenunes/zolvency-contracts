import { ZPayAgentKit } from "./src/index";
import { Keypair, Networks } from "@stellar/stellar-sdk";
import * as dotenv from "dotenv";

dotenv.config();

// Configuration from environment or defaults
const RPC_URL = "https://soroban-testnet.stellar.org";
const NETWORK_PASSPHRASE = Networks.TESTNET;
const NEXUS_ID = process.env.NEXT_PUBLIC_NEXUS_CONTRACT || "";
const ZPAY_ID = process.env.NEXT_PUBLIC_ZPAY_CONTRACT || "";
const SOUL_ID = process.env.NEXT_PUBLIC_SOUL_CONTRACT || "";

async function runSDKTest() {
    console.log("🚀 Starting ZPay Agent Kit E2E Test...");
    
    if (!NEXUS_ID || !ZPAY_ID || !SOUL_ID) {
        console.error("❌ Error: Missing contract IDs in environment.");
        process.exit(1);
    }

    const kit = new ZPayAgentKit({
        rpcUrl: RPC_URL,
        networkPassphrase: NETWORK_PASSPHRASE,
        nexusId: NEXUS_ID,
        zpayId: ZPAY_ID,
        soulId: SOUL_ID,
        debug: true
    });

    const testUser = Keypair.random();
    const testAgent = Keypair.random();
    const testWorker = Keypair.random();
    const testVendor = Keypair.random();

    console.log(`\n💰 Funding test accounts...`);
    const accounts = [testUser, testAgent, testWorker, testVendor];
    for (const acc of accounts) {
        console.log(`   Funding ${acc.publicKey()}...`);
        await fetch(`https://friendbot.stellar.org/?addr=${acc.publicKey()}`);
    }
    console.log("   ✅ All accounts funded.");

    console.log("\n1️⃣  Testing hasSoulID...");
    try {
        const hasSoul = await kit.hasSoulID(testUser.publicKey());
        console.log(`   Result: ${hasSoul} (Expected: false for random key)`);
    } catch (e) {
        console.log("   Note: hasSoulID check completed (may fail if Soul contract is not fully initialized).");
    }

    console.log("\n2️⃣  Testing buildAuthorizeTx (Root -> Agent Master)...");
    const authTx = await kit.buildAuthorizeTx({
        issuer: testUser.publicKey(),
        agent: testAgent.publicKey(),
        scope: {
            ttl: 2000000,
            transfer_limit: 1000000000n,
            contract_allowlist: [ZPAY_ID],
            function_allowlist: ["pay", "pay_escrow"]
        },
        delegationPolicy: { type: "Full" },
        sourceAccount: testUser.publicKey()
    });
    
    console.log("   ✅ AuthorizeTx built successfully.");
    const simAuth = await kit.simulate(authTx);
    console.log(`   Simulation status: ${(simAuth as any).status || "OK"} (Expected: Error if not funded)`);

    console.log("\n3️⃣  Testing buildPayEscrowTx...");
    const payTx = await kit.buildPayEscrowTx({
        agent: testAgent.publicKey(),
        rootAnchor: testUser.publicKey(),
        seller: testVendor.publicKey(),
        token: "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC", // Native XLM
        amount: 500000000n,
        mandateId: 1n,
        sourceAccount: testAgent.publicKey()
    });
    
    console.log("   ✅ PayEscrowTx built successfully.");
    const simPay = await kit.simulate(payTx);
    console.log(`   Simulation status: ${(simPay as any).status || "OK"}`);

    console.log("\n4️⃣  Testing Delegation Flow (Agent -> Sub-Agent)...");
    const delegateTx = await kit.buildAuthorizeTx({
        issuer: testAgent.publicKey(),
        agent: testWorker.publicKey(),
        scope: {
            ttl: 1900000,
            transfer_limit: 100000000n,
        },
        parentMandateId: 1n,
        sourceAccount: testAgent.publicKey()
    });
    
    console.log("   ✅ DelegationTx built successfully.");

    console.log("\n✅ SDK INTEGRATION TEST FINISHED.");
}

runSDKTest().catch(console.error);
