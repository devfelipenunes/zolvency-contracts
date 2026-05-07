"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ZPayAgentKit = exports.ZPayError = void 0;
const stellar_sdk_1 = require("@stellar/stellar-sdk");
/**
 * Custom Error classes for ZPay
 */
class ZPayError extends Error {
    constructor(message, code) {
        super(message);
        this.message = message;
        this.code = code;
        this.name = "ZPayError";
    }
}
exports.ZPayError = ZPayError;
/**
 * ZPayAgentKit: Professional SDK for Agentic Payments.
 */
class ZPayAgentKit {
    constructor(config) {
        this.rpcServer = new stellar_sdk_1.rpc.Server(config.rpcUrl);
        this.networkPassphrase = config.networkPassphrase;
        this.nexusId = config.nexusId;
        this.zpayId = config.zpayId;
        this.soulId = config.soulId;
        this.debug = config.debug || false;
    }
    log(msg) {
        if (this.debug)
            console.log(`[ZPay-SDK] ${msg}`);
    }
    // --- GOVERNANCE METHODS ---
    /**
     * Checks if an address has a valid SoulID.
     * Mandatory for any mandate issuance.
     */
    async hasSoulID(address) {
        this.log(`Checking SoulID for ${address}...`);
        const contract = new stellar_sdk_1.Contract(this.soulId);
        const res = await this.rpcServer.getLedgerEntries(contract.getFootprint());
        // Simplified: using simulation to check
        const dummyAccount = {
            accountId: () => "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
            sequenceNumber: () => "0",
            incrementSequenceNumber: () => { }
        };
        const tx = new stellar_sdk_1.TransactionBuilder(dummyAccount, { fee: "100", networkPassphrase: this.networkPassphrase })
            .addOperation(contract.call("has_soul", (0, stellar_sdk_1.nativeToScVal)(address, { type: "address" })))
            .setTimeout(30)
            .build();
        const sim = await this.rpcServer.simulateTransaction(tx);
        if (stellar_sdk_1.rpc.Api.isSimulationSuccess(sim)) {
            return (0, stellar_sdk_1.scValToNative)(sim.result.retval);
        }
        return false;
    }
    /**
     * Build transaction to Issue a Mandate
     */
    async buildAuthorizeTx(params) {
        this.log(`Building AuthorizeTx for agent ${params.agent}`);
        const contract = new stellar_sdk_1.Contract(this.nexusId);
        const account = await this.rpcServer.getAccount(params.sourceAccount);
        let policyVal;
        const policy = params.delegationPolicy || { type: "None" };
        if (policy.type === "None")
            policyVal = (0, stellar_sdk_1.nativeToScVal)(0, { type: "u32" });
        else if (policy.type === "Full")
            policyVal = (0, stellar_sdk_1.nativeToScVal)(1, { type: "u32" });
        else
            policyVal = (0, stellar_sdk_1.nativeToScVal)({ Restricted: policy.rules });
        const op = contract.call("issue_mandate", (0, stellar_sdk_1.nativeToScVal)(params.issuer, { type: "address" }), (0, stellar_sdk_1.nativeToScVal)(params.agent, { type: "address" }), (0, stellar_sdk_1.nativeToScVal)({
            ttl: params.scope.ttl,
            transfer_limit: params.scope.transfer_limit || null,
            scope_commitment: null,
            contract_allowlist: params.scope.contract_allowlist || null,
            function_allowlist: params.scope.function_allowlist || null
        }), policyVal, (0, stellar_sdk_1.nativeToScVal)(params.parentMandateId || null));
        return this.createTx(account, op);
    }
    // --- PAYMENT METHODS ---
    /**
     * Immediate payment via ZPay
     */
    async pay(params) {
        const contract = new stellar_sdk_1.Contract(this.zpayId);
        const account = await this.rpcServer.getAccount(params.sourceAccount);
        const op = contract.call("pay", (0, stellar_sdk_1.nativeToScVal)(params.agent, { type: "address" }), (0, stellar_sdk_1.nativeToScVal)(params.rootAnchor, { type: "address" }), (0, stellar_sdk_1.nativeToScVal)(params.seller, { type: "address" }), (0, stellar_sdk_1.nativeToScVal)(params.token, { type: "address" }), (0, stellar_sdk_1.nativeToScVal)(params.amount, { type: "i128" }), (0, stellar_sdk_1.nativeToScVal)(params.mandateId, { type: "u64" }), (0, stellar_sdk_1.nativeToScVal)(params.priceTicket || null), (0, stellar_sdk_1.nativeToScVal)(null));
        const tx = this.createTx(account, op);
        if (params.signer) {
            tx.sign(params.signer);
            return this.sendTx(tx);
        }
        return tx;
    }
    /**
     * Lock funds in Escrow
     */
    async payEscrow(params) {
        const contract = new stellar_sdk_1.Contract(this.zpayId);
        const account = await this.rpcServer.getAccount(params.sourceAccount);
        const op = contract.call("pay_escrow", (0, stellar_sdk_1.nativeToScVal)(params.agent, { type: "address" }), (0, stellar_sdk_1.nativeToScVal)(params.rootAnchor, { type: "address" }), (0, stellar_sdk_1.nativeToScVal)(params.seller, { type: "address" }), (0, stellar_sdk_1.nativeToScVal)(params.token, { type: "address" }), (0, stellar_sdk_1.nativeToScVal)(params.amount, { type: "i128" }), (0, stellar_sdk_1.nativeToScVal)(params.mandateId, { type: "u64" }), (0, stellar_sdk_1.nativeToScVal)(params.priceTicket || null), (0, stellar_sdk_1.nativeToScVal)(null));
        const tx = this.createTx(account, op);
        if (params.signer) {
            tx.sign(params.signer);
            return this.sendTx(tx);
        }
        return tx;
    }
    /**
     * Release Escrow
     */
    async releaseEscrow(params) {
        const contract = new stellar_sdk_1.Contract(this.zpayId);
        const account = await this.rpcServer.getAccount(params.sourceAccount);
        const op = contract.call("release_escrow", (0, stellar_sdk_1.nativeToScVal)(params.caller, { type: "address" }), (0, stellar_sdk_1.nativeToScVal)(params.paymentId, { type: "u64" }));
        const tx = this.createTx(account, op);
        if (params.signer) {
            tx.sign(params.signer);
            return this.sendTx(tx);
        }
        return tx;
    }
    // --- UTILITIES ---
    createTx(account, op) {
        return new stellar_sdk_1.TransactionBuilder(account, {
            fee: stellar_sdk_1.BASE_FEE,
            networkPassphrase: this.networkPassphrase,
        })
            .addOperation(op)
            .setTimeout(30)
            .build();
    }
    async sendTx(tx) {
        this.log(`Sending transaction ${tx.hash().toString("hex")}...`);
        const response = await this.rpcServer.sendTransaction(tx);
        if (response.status === "PENDING") {
            return this.pollForResult(response.hash);
        }
        throw new ZPayError(`Transaction failed: ${response.status}`, response.status);
    }
    async pollForResult(hash) {
        let attempts = 0;
        while (attempts < 15) {
            const res = await this.rpcServer.getTransaction(hash);
            if (res.status === "SUCCESS")
                return res;
            if (res.status === "FAILED")
                throw new ZPayError("Transaction execution failed", "FAILED");
            await new Promise(r => setTimeout(r, 1000));
            attempts++;
        }
        throw new ZPayError("Transaction timeout", "TIMEOUT");
    }
}
exports.ZPayAgentKit = ZPayAgentKit;
