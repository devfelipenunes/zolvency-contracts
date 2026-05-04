const { rpc, Keypair, Asset, Operation, TransactionBuilder, Networks, xdr, nativeToScVal, Address } = require('@stellar/stellar-sdk');
require('dotenv').config();

const server = new rpc.Server('https://soroban-testnet.stellar.org:443');
const networkPassphrase = Networks.TESTNET;

async function run() {
    const deployer = Keypair.fromSecret(process.env.DEPLOYER_SECRET);
    const admin = process.env.ADMIN_PUBLIC;
    
    // Use the Adapter ID from a previous run or deploy a new one
    // Let's use the one from the last failed run: CC33YJYVV7AQPHPMTGAIGACM4YVS72SUT6M6EWUWI2TXMWQZ3FNL72SP
    // Actually, I'll deploy a fresh one to be sure it's initialized correctly.
    const adapterWasm = 'target/wasm32-unknown-unknown/release/zolvency_axelar_adapter.optimized.wasm';
    
    console.log('1. Deploying/Using Adapter...');
    // For speed, let's just use a known one or deploy
    // I'll use the CLI to deploy it first to save JS boilerplate
}
run();
