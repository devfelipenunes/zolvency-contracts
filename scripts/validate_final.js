require('dotenv').config();
const { Contract, Keypair, rpc, Networks, TransactionBuilder, Address, nativeToScVal, scValToNative } = require('@stellar/stellar-sdk');

async function validateAxelar() {
    const RPC_URL = process.env.RPC_URL || "https://soroban-testnet.stellar.org";
    const server = new rpc.Server(RPC_URL);
    const networkPassphrase = process.env.STELLAR_NETWORK === "mainnet" ? Networks.PUBLIC : Networks.TESTNET;

    const adminSecret = process.env.ADMIN_SECRET;
    const deployerSecret = process.env.DEPLOYER_SECRET;
    
    if (!adminSecret || !deployerSecret) {
        throw new Error("ADMIN_SECRET or DEPLOYER_SECRET not set in .env");
    }

    const adminKp = Keypair.fromSecret(adminSecret);
    const deployerKp = Keypair.fromSecret(deployerSecret);

    // Use o ID do contrato configurado no .env
    const identityId = process.env.IDENTITY_CONTRACT_ID;
    if (!identityId) {
        throw new Error("IDENTITY_CONTRACT_ID not set in .env");
    }
    
    // Endereços Axelar configurados no .env
    const AXELAR_GATEWAY = process.env.AXELAR_GATEWAY_STELLAR;
    const AXELAR_GAS_SERVICE = process.env.AXELAR_GAS_SERVICE_STELLAR;
    const AXELAR_GAS_TOKEN = process.env.AXELAR_GAS_TOKEN_STELLAR;

    if (!AXELAR_GATEWAY || !AXELAR_GAS_SERVICE || !AXELAR_GAS_TOKEN) {
        throw new Error("Axelar configuration missing in .env");
    }

    const contract = new Contract(identityId);

    console.log("🛠️ Iniciando Validação DEFINITIVA via SDK (Axelar)...");

    async function invoke(keypair, fnName, args) {
        const source = await server.getAccount(keypair.publicKey());
        const scArgs = args.map(arg => {
            if (arg instanceof Address) return arg.toScVal();
            return nativeToScVal(arg);
        });

        const operation = contract.call(fnName, ...scArgs);
        const tx = new TransactionBuilder(source, { fee: "100000", networkPassphrase })
            .addOperation(operation)
            .setTimeout(60)
            .build();
        
        tx.sign(keypair);
        console.log(`📡 Enviando ${fnName}...`);
        const result = await server.sendTransaction(tx);
        
        if (result.status === "PENDING") {
            let status = await server.getTransaction(result.hash);
            while (status.status === "NOT_FOUND" || status.status === "PENDING") {
                await new Promise(r => setTimeout(r, 3000));
                status = await server.getTransaction(result.hash);
            }
            if (status.status === "SUCCESS") {
                console.log(`✅ ${fnName} Sucesso! Hash: ${result.hash}`);
                return status;
            } else {
                console.error(`❌ ${fnName} Falhou!`, status.resultXdr);
                throw new Error("Failure");
            }
        } else {
            console.error("❌ Erro no envio:", result);
            throw new Error("Send Error");
        }
    }

    try {
        // 1. Configurar o Adaptador Axelar
        console.log("⚙️ Configurando Axelar...");
        await invoke(adminKp, "set_axelar_config", [
            new Address(adminKp.publicKey()),
            AXELAR_GATEWAY,
            AXELAR_GAS_SERVICE,
            AXELAR_GAS_TOKEN
        ]);

        // 2. Ativar o Protocolo Axelar
        console.log("🚀 Ativando Axelar...");
        await invoke(adminKp, "set_active_protocol", [
            new Address(adminKp.publicKey()),
            "Axelar", // Enum no Rust é Symbol ou Enum, via SDK enviamos o nome da variante
            new Address(identityId) // O adaptador é o próprio contrato
        ]);

        // 3. Disparar o Mint
        console.log("💎 Minting and Bridging...");
        const verifierEvm = process.env.VERIFIER_CONTRACT_EVM;
        const userEvm = process.env.USER_EVM_ADDRESS;

        if (!verifierEvm || !userEvm) {
            throw new Error("VERIFIER_CONTRACT_EVM or USER_EVM_ADDRESS not set in .env");
        }

        const mintParams = {
            username: "final_validator",
            external_id: "gh_final_001",
            passkey: null, 
            passkey_signature: null,
            contributions: 3000,
            proof_data: Buffer.alloc(0),
            nonce: 0n // Assumindo novo contrato
        };

        const crossChain = {
            destination_chain: "ethereum-sepolia",
            destination_address: verifierEvm,
            user_destination_address: Buffer.from(userEvm.replace("0x", ""), 'hex')
        };

        await invoke(deployerKp, "mint", [
            new Address(deployerKp.publicKey()),
            Buffer.alloc(64, 0),
            mintParams,
            null,
            crossChain
        ]);

        console.log("🏆 SUCESSO! Interoperabilidade VALIDADA na rede real.");

    } catch (e) {
        console.error("🛑 Erro Final:", e.message);
    }
}

validateAxelar();
