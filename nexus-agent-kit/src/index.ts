import { 
    Address, 
    rpc, 
    Networks, 
    TransactionBuilder, 
    xdr, 
    Contract, 
    nativeToScVal, 
    scValToNative,
    BASE_FEE,
    Transaction,
    Keypair,
    FeeBumpTransaction
} from "@stellar/stellar-sdk";

/**
 * Custom Error classes for ZPay
 */
export class ZPayError extends Error {
    constructor(public message: string, public code?: string) {
        super(message);
        this.name = "ZPayError";
    }
}

/**
 * Types for the Zolvency Ecosystem
 */
export type DelegationPolicy = 
    | { type: "None" }
    | { type: "Full" }
    | { type: "Restricted", rules: { max_subdepth: number; budget_fraction?: number } };

export interface MandateScope {
    ttl: number;
    transfer_limit?: bigint;
    renewal_period?: bigint; // Seconds (e.g. 2.592M for 30 days)
    contract_allowlist?: string[];
    function_allowlist?: string[];
}

export interface PriceTicket {
    base_currency: string;
    price_per_unit: bigint;
    signature: string;
    timestamp: number;
}

/**
 * ZPayAgentKit: Professional SDK for Agentic Payments.
 */
export class ZPayAgentKit {
    private rpcServer: rpc.Server;
    private networkPassphrase: string;
    private nexusId: string;
    private zpayId: string;
    private soulId: string;
    private debug: boolean;

    constructor(config: {
        rpcUrl: string;
        networkPassphrase: string;
        nexusId: string;
        zpayId: string;
        soulId: string;
        debug?: boolean;
    }) {
        this.rpcServer = new rpc.Server(config.rpcUrl);
        this.networkPassphrase = config.networkPassphrase;
        this.nexusId = config.nexusId;
        this.zpayId = config.zpayId;
        this.soulId = config.soulId;
        this.debug = config.debug || false;
    }

    private log(msg: string) {
        if (this.debug) console.log(`[ZPay-SDK] ${msg}`);
    }

    // --- GOVERNANCE METHODS ---

    /**
     * Checks if an address has a valid SoulID.
     * Mandatory for any mandate issuance.
     */
    async hasSoulID(address: string): Promise<boolean> {
        this.log(`Checking SoulID for ${address}...`);
        const contract = new Contract(this.soulId);
        const res = await this.rpcServer.getLedgerEntries(
            contract.getFootprint()
        );
        // Simplified: using simulation to check
        const dummyAccount = {
            accountId: () => "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
            sequenceNumber: () => "0",
            incrementSequenceNumber: () => {}
        };
        const tx = new TransactionBuilder(
            dummyAccount as any,
            { fee: "100", networkPassphrase: this.networkPassphrase }
        )
        .addOperation(contract.call("has_soul", nativeToScVal(address, { type: "address" })))
        .setTimeout(30)
        .build();

        const sim = await this.rpcServer.simulateTransaction(tx);
        if (rpc.Api.isSimulationSuccess(sim)) {
            return scValToNative(sim.result!.retval);
        }
        return false;
    }

    /**
     * Build transaction to Issue a Mandate
     */
    async buildAuthorizeTx(params: {
        issuer: string;
        agent: string;
        scope: MandateScope;
        delegationPolicy?: DelegationPolicy;
        parentMandateId?: bigint;
        sourceAccount: string;
    }): Promise<Transaction> {
        this.log(`Building AuthorizeTx for agent ${params.agent}`);
        const contract = new Contract(this.nexusId);
        const account = await this.rpcServer.getAccount(params.sourceAccount);
        
        let policyVal;
        const policy = params.delegationPolicy || { type: "None" };
        if (policy.type === "None") policyVal = nativeToScVal(0, { type: "u32" });
        else if (policy.type === "Full") policyVal = nativeToScVal(1, { type: "u32" });
        else policyVal = nativeToScVal({ Restricted: policy.rules });

        const op = contract.call("issue_mandate", 
            nativeToScVal(params.issuer, { type: "address" }),
            nativeToScVal(params.agent, { type: "address" }),
            nativeToScVal({
                ttl: params.scope.ttl,
                transfer_limit: params.scope.transfer_limit || null,
                renewal_period: params.scope.renewal_period || null,
                scope_commitment: null,
                contract_allowlist: params.scope.contract_allowlist || null,
                function_allowlist: params.scope.function_allowlist || null
            }),
            policyVal,
            nativeToScVal(params.parentMandateId || null)
        );

        return this.createTx(account, op);
    }

    // --- PAYMENT METHODS ---

    /**
     * Immediate payment via ZPay
     */
    async buildPayTx(params: {
        agent: string;
        rootAnchor: string;
        seller: string;
        token: string;
        amount: bigint;
        mandateId: bigint;
        priceTicket?: PriceTicket;
        sourceAccount: string;
    }): Promise<Transaction> {
        const contract = new Contract(this.zpayId);
        const account = await this.rpcServer.getAccount(params.sourceAccount);

        const op = contract.call("pay",
            nativeToScVal(params.agent, { type: "address" }),
            nativeToScVal(params.rootAnchor, { type: "address" }),
            nativeToScVal(params.seller, { type: "address" }),
            nativeToScVal(params.token, { type: "address" }),
            nativeToScVal(params.amount, { type: "i128" }),
            nativeToScVal(params.mandateId, { type: "u64" }),
            nativeToScVal(params.priceTicket || null),
            nativeToScVal(null)
        );

        return this.createTx(account, op);
    }

    /**
     * Lock funds in Escrow
     */
    async buildPayEscrowTx(params: {
        agent: string;
        rootAnchor: string;
        seller: string;
        token: string;
        amount: bigint;
        mandateId: bigint;
        priceTicket?: PriceTicket;
        sourceAccount: string;
    }): Promise<Transaction> {
        const contract = new Contract(this.zpayId);
        const account = await this.rpcServer.getAccount(params.sourceAccount);

        const op = contract.call("pay_escrow",
            nativeToScVal(params.agent, { type: "address" }),
            nativeToScVal(params.rootAnchor, { type: "address" }),
            nativeToScVal(params.seller, { type: "address" }),
            nativeToScVal(params.token, { type: "address" }),
            nativeToScVal(params.amount, { type: "i128" }),
            nativeToScVal(params.mandateId, { type: "u64" }),
            nativeToScVal(params.priceTicket || null),
            nativeToScVal(null)
        );

        return this.createTx(account, op);
    }

    /**
     * Release Escrow
     */
    async buildReleaseEscrowTx(params: {
        caller: string;
        paymentId: bigint;
        sourceAccount: string;
    }): Promise<Transaction> {
        const contract = new Contract(this.zpayId);
        const account = await this.rpcServer.getAccount(params.sourceAccount);
        const op = contract.call("release_escrow",
            nativeToScVal(params.caller, { type: "address" }),
            nativeToScVal(params.paymentId, { type: "u64" })
        );
        return this.createTx(account, op);
    }

    /**
     * Execute a pull payment (subscription charge) as an authorized seller.
     */
    async buildChargeSubscriptionTx(params: {
        seller: string;
        rootAnchor: string;
        token: string;
        amount: bigint;
        mandateId: bigint;
        priceTicket?: PriceTicket;
        sourceAccount: string;
    }): Promise<Transaction> {
        const contract = new Contract(this.zpayId);
        const account = await this.rpcServer.getAccount(params.sourceAccount);

        const op = contract.call("charge_subscription",
            nativeToScVal(params.seller, { type: "address" }),
            nativeToScVal(params.rootAnchor, { type: "address" }),
            nativeToScVal(params.token, { type: "address" }),
            nativeToScVal(params.amount, { type: "i128" }),
            nativeToScVal(params.mandateId, { type: "u64" }),
            nativeToScVal(params.priceTicket || null),
            nativeToScVal(null)
        );

        return this.createTx(account, op);
    }

    /**
     * Wrap a transaction with a Fee Bump. 
     * Allows a sponsor (e.g., User) to pay gas for an Agent.
     */
    buildFeeBumpTx(params: {
        innerTx: Transaction;
        feeSource: string;
        baseFee?: number;
    }): Transaction | FeeBumpTransaction {
        return TransactionBuilder.buildFeeBumpTransaction(
            Keypair.fromPublicKey(params.feeSource),
            (params.baseFee || BASE_FEE).toString(),
            params.innerTx,
            this.networkPassphrase
        );
    }

    /**
     * Fetches transaction history for a specific mandate from ledger events.
     */
    async fetchAgentHistory(mandateId: bigint): Promise<any[]> {
        const response = await this.rpcServer.getEvents({
            startLedger: 0,
            filters: [
                {
                    type: "contract",
                    contractIds: [this.zpayId],
                    topics: [["*", nativeToScVal(mandateId, { type: "u64" }).toXDR("base64")]]
                }
            ]
        });
        return response.events.map(e => ({
            id: e.id,
            ledger: e.ledger,
            data: scValToNative(e.value)
        }));
    }

    // --- UTILITIES ---

    private createTx(account: any, op: xdr.Operation): Transaction {
        return new TransactionBuilder(account, {
            fee: BASE_FEE,
            networkPassphrase: this.networkPassphrase,
        })
        .addOperation(op)
        .setTimeout(30)
        .build();
    }

    async sendTx(tx: Transaction): Promise<rpc.Api.GetTransactionResponse> {
        this.log(`Sending transaction ${tx.hash().toString("hex")}...`);
        const response = await this.rpcServer.sendTransaction(tx);
        if (response.status === "PENDING") {
            return this.pollForResult(response.hash);
        }
        throw new ZPayError(`Transaction failed: ${response.status}`, response.status);
    }

    async simulate(tx: Transaction): Promise<rpc.Api.SimulateTransactionResponse> {
        this.log(`Simulating transaction...`);
        return this.rpcServer.simulateTransaction(tx);
    }

    private async pollForResult(hash: string): Promise<rpc.Api.GetTransactionResponse> {
        let attempts = 0;
        while (attempts < 15) {
            const res = await this.rpcServer.getTransaction(hash);
            if (res.status === "SUCCESS") return res;
            if (res.status === "FAILED") throw new ZPayError("Transaction execution failed", "FAILED");
            await new Promise(r => setTimeout(r, 1000));
            attempts++;
        }
        throw new ZPayError("Transaction timeout", "TIMEOUT");
    }
}
