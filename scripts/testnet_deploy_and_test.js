const { 
    Asset, 
    Keypair, 
    Networks, 
    Operation, 
    TransactionBuilder, 
    rpc, 
    nativeToScVal, 
    scValToNative, 
    Address,
    xdr
} = require('@stellar/stellar-sdk');
const fs = require('fs');
const crypto = require('crypto');
require('dotenv').config();

const server = new rpc.Server(process.env.RPC_URL);
const networkPassphrase = Networks.TESTNET;

const deployerKeypair = Keypair.fromSecret(process.env.DEPLOYER_SECRET);
const deployerAddress = deployerKeypair.publicKey();

async function submitTx(transaction) {
    transaction.sign(deployerKeypair);
    const response = await server.sendTransaction(transaction);
    if (response.status !== 'PENDING') {
        throw new Error(`Tx failed: ${JSON.stringify(response)}`);
    }
    
    let status = 'PENDING';
    let txResponse;
    let retries = 0;
    while (status === 'PENDING' || (status === 'NOT_FOUND' && retries < 10)) {
        await new Promise(r => setTimeout(r, 2000));
        txResponse = await server.getTransaction(response.hash);
        status = txResponse.status;
        retries++;
        console.log(`Checking tx ${response.hash}... Status: ${status}`);
    }
    
    if (status !== 'SUCCESS') {
        throw new Error(`Tx failed: ${JSON.stringify(txResponse)}`);
    }
    return txResponse;
}

function decodeMeta(meta) {
    if (typeof meta === 'string') {
        return xdr.TransactionMeta.fromXDR(Buffer.from(meta, 'base64'));
    }
    return meta;
}

async function uploadWasm(wasmPath) {
    const wasm = fs.readFileSync(wasmPath);
    console.log(`Uploading ${wasmPath}...`);
    
    const account = await server.getAccount(deployerAddress);
    const tx = new TransactionBuilder(account, { fee: '1000000', networkPassphrase })
        .addOperation(Operation.invokeHostFunction({
            func: xdr.HostFunction.hostFunctionTypeUploadContractWasm(wasm),
            auth: []
        }))
        .setTimeout(30)
        .build();

    const preparedTx = await server.prepareTransaction(tx);
    const result = await submitTx(preparedTx);
    
    if (result.returnValue) {
        const wasmHash = result.returnValue.bytes().toString('hex');
        console.log(`Wasm uploaded. Hash: ${wasmHash}`);
        return wasmHash;
    }
    
    const wasmHash = decodeMeta(result.resultMetaXdr)
        .v3().sorobanMeta().returnValue().bytes().toString('hex');
    console.log(`Wasm uploaded. Hash: ${wasmHash}`);
    return wasmHash;
}

async function createContract(wasmHash) {
    console.log(`Creating contract for hash ${wasmHash}...`);
    const account = await server.getAccount(deployerAddress);
    
    const tx = new TransactionBuilder(account, { fee: '1000000', networkPassphrase })
        .addOperation(Operation.invokeHostFunction({
            func: xdr.HostFunction.hostFunctionTypeCreateContract(new xdr.CreateContractArgs({
                contractIdPreimage: xdr.ContractIdPreimage.contractIdPreimageFromAddress(
                    new xdr.ContractIdPreimageFromAddress({
                        address: Address.fromString(deployerAddress).toScAddress(),
                        salt: crypto.randomBytes(32)
                    })
                ),
                executable: xdr.ContractExecutable.contractExecutableWasm(Buffer.from(wasmHash, 'hex'))
            })),
            auth: []
        }))
        .setTimeout(30)
        .build();

    const preparedTx = await server.prepareTransaction(tx);
    const result = await submitTx(preparedTx);
    
    if (result.returnValue) {
        const contractId = Address.fromScAddress(result.returnValue.address()).toString();
        console.log(`Contract created. ID: ${contractId}`);
        return contractId;
    }

    const contractId = Address.fromScAddress(
        decodeMeta(result.resultMetaXdr)
            .v3().sorobanMeta().returnValue().address()
    ).toString();
    console.log(`Contract created. ID: ${contractId}`);
    return contractId;
}

async function invoke(contractId, functionName, args) {
    console.log(`Invoking ${functionName} on ${contractId}...`);
    const account = await server.getAccount(deployerAddress);
    
    const tx = new TransactionBuilder(account, { fee: '1000000', networkPassphrase })
        .addOperation(Operation.invokeContractFunction({
            contractId,
            functionName,
            args
        }))
        .setTimeout(30)
        .build();

    const preparedTx = await server.prepareTransaction(tx);
    const result = await submitTx(preparedTx);

    if (result.returnValue) {
        return result.returnValue;
    }

    return decodeMeta(result.resultMetaXdr)
        .v3().sorobanMeta().returnValue();
}

async function run() {
    try {
        // 1. Upload & Create
        const soulWasm = await uploadWasm('target/wasm32-unknown-unknown/release/zolvency_soul.optimized.wasm');
        const soulId = await createContract(soulWasm);
        
        const registryWasm = await uploadWasm('target/wasm32-unknown-unknown/release/zolvency_registry.optimized.wasm');
        const registryId = await createContract(registryWasm);
        
        const githubWasm = await uploadWasm('target/wasm32-unknown-unknown/release/github_identity.optimized.wasm');
        const githubId = await createContract(githubWasm);
        
        const adapterWasm = await uploadWasm('target/wasm32-unknown-unknown/release/zolvency_axelar_adapter.wasm');
        const adapterId = await createContract(adapterWasm);

        // 2. Initialize
        console.log('Initializing contracts with addresses:', {
            deployerAddress,
            soulId,
            registryId,
            githubId,
            adapterId,
            gateway: process.env.AXELAR_GATEWAY_STELLAR,
            gasService: process.env.AXELAR_GAS_SERVICE_STELLAR,
            gasToken: process.env.AXELAR_GAS_TOKEN_STELLAR,
            treasury: process.env.TREASURY_ADDRESS
        });

        await invoke(soulId, 'initialize', [
            Address.fromString(deployerAddress).toScVal(),
            Address.fromString(deployerAddress).toScVal()
        ]);

        await invoke(registryId, 'initialize', [
            Address.fromString(deployerAddress).toScVal(),
            Address.fromString(deployerAddress).toScVal()
        ]);

        await invoke(adapterId, 'initialize', [
            Address.fromString(deployerAddress).toScVal(),
            Address.fromString(soulId).toScVal(),
            Address.fromString(process.env.AXELAR_GATEWAY_STELLAR).toScVal(),
            Address.fromString(process.env.AXELAR_GAS_SERVICE_STELLAR).toScVal(),
            Address.fromString(process.env.AXELAR_GAS_TOKEN_STELLAR).toScVal()
        ]);

        // 3. Configure
        await invoke(registryId, 'set_interop_config', [
            Address.fromString(deployerAddress).toScVal(),
            xdr.ScVal.scvMap([
                new xdr.ScMapEntry({
                    key: nativeToScVal('active_protocol'),
                    val: xdr.ScVal.scvSymbol('Axelar')
                }),
                new xdr.ScMapEntry({
                    key: nativeToScVal('adapter_address'),
                    val: Address.fromString(adapterId).toScVal()
                })
            ])
        ]);

        await invoke(githubId, 'initialize', [
            Address.fromString(deployerAddress).toScVal(),
            Address.fromString(registryId).toScVal(),
            Address.fromString(soulId).toScVal(),
            Address.fromString(process.env.AXELAR_GAS_TOKEN_STELLAR).toScVal(),
            Address.fromString(deployerAddress).toScVal(),
            Address.fromString(process.env.TREASURY_ADDRESS).toScVal(),
            nativeToScVal(0, { type: 'i128' })
        ]);

        await invoke(registryId, 'register_token', [
            Address.fromString(deployerAddress).toScVal(),
            Address.fromString(githubId).toScVal()
        ]);

        // 4. Mint Soul
        await invoke(soulId, 'mint', [
            Address.fromString(deployerAddress).toScVal(),
            nativeToScVal(Buffer.alloc(64), { type: 'bytes' }),
            nativeToScVal(Buffer.alloc(64), { type: 'bytes' })
        ]);

        // 5. Cross-chain Mint
        console.log("🚀 Executing Cross-chain Mint...");
        // Use a dummy verifier address for now if not deployed yet, or use the one from .env if updated
        const verifierAddress = "0x71e067692691c3A1c53D4Ab126BbEA76162BFD06"; 

        await invoke(githubId, 'mint', [
            Address.fromString(deployerAddress).toScVal(),
            nativeToScVal(1, { type: 'u32' }),
            nativeToScVal({
                username: 'testuser',
                external_id: 'test_123',
                contributions: 1500,
                proof_data: Buffer.alloc(0),
                nonce: 0
            }),
            nativeToScVal({
                destination_chain: 'ethereum-sepolia',
                destination_address: verifierAddress,
                user_destination_address: Buffer.from('71e067692691c3A1c53D4Ab126BbEA76162BFD06', 'hex')
            })
        ]);

        console.log("✅ All steps completed successfully!");
        console.log(`Registry ID: ${registryId}`);
        console.log(`Github ID: ${githubId}`);
        console.log(`Adapter ID: ${adapterId}`);

    } catch (e) {
        console.error("❌ Error during test:", e);
        if (e.data && e.data.extras && e.data.extras.result_codes) {
            console.error("Result codes:", e.data.extras.result_codes);
        }
    }
}

run();
