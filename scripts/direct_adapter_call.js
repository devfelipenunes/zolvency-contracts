const { rpc, Keypair, Networks, xdr, nativeToScVal, Address, TransactionBuilder, Operation } = require('@stellar/stellar-sdk');
require('dotenv').config();

const server = new rpc.Server('https://soroban-testnet.stellar.org:443');

async function run() {
    const deployer = Keypair.fromSecret(process.env.DEPLOYER_SECRET);
    const admin = process.env.ADMIN_PUBLIC;
    
    // Using the Adapter already deployed in previous step to save time
    const adapterId = 'CAT6WGDW245SS53KAO7C2C7I4AADMIRGN4MD5ZW6UWJT57GKD6XRN4F3';
    
    console.log('Fetching account...');
    const account = await server.getAccount(deployer.publicKey());
    
    console.log('Building transaction...');
    // We call send_reputation(caller, dest_chain, dest_addr, ext_id, tier, user_addr, nonce, token_type, ecosystem)
    const tx = new TransactionBuilder(account, { fee: '100000', networkPassphrase: Networks.TESTNET })
        .addOperation(Operation.invokeContractFunction({
            contract: adapterId,
            function: 'send_reputation',
            args: [
                nativeToScVal(admin, { type: 'address' }),
                nativeToScVal('ethereum-sepolia'),
                nativeToScVal('0x71e067692691c3A1c53D4Ab126BbEA76162BFD06'),
                nativeToScVal('gh_direct_js'),
                nativeToScVal(1, { type: 'u32' }),
                nativeToScVal(Buffer.alloc(0), { type: 'bytes' }),
                nativeToScVal(0, { type: 'u64' }),
                nativeToScVal('github', { type: 'symbol' }),
                xdr.ScVal.scvVec([xdr.ScVal.scvSymbol('Evm')]) // Enum Ecosystem::Evm
            ]
        }))
        .setTimeout(30)
        .build();

    console.log('Simulating...');
    const simulation = await server.simulateTransaction(tx);
    if (rpc.Api.isSimulationError(simulation)) {
        console.error('Simulation failed:', JSON.stringify(simulation, null, 2));
        return;
    }
    
    console.log('Simulation success! Fee:', simulation.minResourceFee);
    
    tx.setSorobanFee(simulation.minResourceFee);
    tx.setSorobanData(simulation.transactionData);
    tx.sign(deployer);
    
    console.log('Submitting...');
    const response = await server.sendTransaction(tx);
    console.log('Response:', response.hash);
}

run().catch(console.error);
