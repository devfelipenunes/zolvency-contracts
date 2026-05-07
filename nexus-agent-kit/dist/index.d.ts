import { rpc, Transaction, Keypair } from "@stellar/stellar-sdk";
/**
 * Custom Error classes for ZPay
 */
export declare class ZPayError extends Error {
    message: string;
    code?: string | undefined;
    constructor(message: string, code?: string | undefined);
}
/**
 * Types for the Zolvency Ecosystem
 */
export type DelegationPolicy = {
    type: "None";
} | {
    type: "Full";
} | {
    type: "Restricted";
    rules: {
        max_subdepth: number;
        budget_fraction?: number;
    };
};
export interface MandateScope {
    ttl: number;
    transfer_limit?: bigint;
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
export declare class ZPayAgentKit {
    private rpcServer;
    private networkPassphrase;
    private nexusId;
    private zpayId;
    private soulId;
    private debug;
    constructor(config: {
        rpcUrl: string;
        networkPassphrase: string;
        nexusId: string;
        zpayId: string;
        soulId: string;
        debug?: boolean;
    });
    private log;
    /**
     * Checks if an address has a valid SoulID.
     * Mandatory for any mandate issuance.
     */
    hasSoulID(address: string): Promise<boolean>;
    /**
     * Build transaction to Issue a Mandate
     */
    buildAuthorizeTx(params: {
        issuer: string;
        agent: string;
        scope: MandateScope;
        delegationPolicy?: DelegationPolicy;
        parentMandateId?: bigint;
        sourceAccount: string;
    }): Promise<Transaction>;
    /**
     * Immediate payment via ZPay
     */
    pay(params: {
        agent: string;
        rootAnchor: string;
        seller: string;
        token: string;
        amount: bigint;
        mandateId: bigint;
        priceTicket?: PriceTicket;
        sourceAccount: string;
        signer?: Keypair;
    }): Promise<rpc.Api.GetTransactionResponse | Transaction>;
    /**
     * Lock funds in Escrow
     */
    payEscrow(params: {
        agent: string;
        rootAnchor: string;
        seller: string;
        token: string;
        amount: bigint;
        mandateId: bigint;
        priceTicket?: PriceTicket;
        sourceAccount: string;
        signer?: Keypair;
    }): Promise<rpc.Api.GetTransactionResponse | Transaction>;
    /**
     * Release Escrow
     */
    releaseEscrow(params: {
        caller: string;
        paymentId: bigint;
        sourceAccount: string;
        signer?: Keypair;
    }): Promise<rpc.Api.GetTransactionResponse | Transaction>;
    private createTx;
    sendTx(tx: Transaction): Promise<rpc.Api.GetTransactionResponse>;
    private pollForResult;
}
