# ZPay Agent Kit - API Reference (v2.1)

The **ZPay Agent Kit** is a professional-grade SDK for building autonomous agents on the Zolvency ecosystem. It manages identity (SoulID), governance (Nexus), and payments (ZPay) in a single, high-level interface.

---

## Initialization

```typescript
import { ZPayAgentKit } from "@zolvency/zpay-agent-kit";

const kit = new ZPayAgentKit({
    rpcUrl: "https://soroban-testnet.stellar.org",
    networkPassphrase: "Test SDF Network ; September 2015",
    nexusId: "CBDETDV...",
    zpayId: "CAHQHNS...",
    soulId: "CCXMGVH...",
    debug: true // Enable detailed logs of on-chain actions
});
```

---

## Identity & Governance (Nexus)

### `hasSoulID`
Checks if an address owns a valid SoulID.
- `address`: (string) The address to check.
- **Returns**: `Promise<boolean>`

### `buildAuthorizeTx`
Builds a transaction to issue a Mandate (Authorize an Agent).
- `issuer`: (string) Source of authority.
- `agent`: (string) Target agent address.
- `scope`: ([MandateScope](#mandatescope)) Limitations.
- `delegationPolicy`: ([DelegationPolicy](#delegationpolicy)) Optional.
- `parentMandateId`: (bigint) Optional (for sub-agents).
- `sourceAccount`: (string) Transaction source.
- **Returns**: `Promise<Transaction>`

---

## Payments (ZPay)

All payment methods support an optional **Signer Pattern**. If a `signer` (Keypair) is provided, the SDK will build, sign, send, and poll for the transaction result automatically.

### `pay` (Immediate)
Builds or executes a direct, immediate payment.
- `params`:
  - ...all standard payment fields (agent, rootAnchor, seller, token, amount, mandateId).
  - `signer`: (Keypair) Optional. If provided, returns `GetTransactionResponse`.
- **Returns**: `Promise<Transaction | GetTransactionResponse>`

### `payEscrow` (Lock)
Locks funds into the ZPay Trustless Vault.
- `params`: Same as `pay`.
- **Returns**: `Promise<Transaction | GetTransactionResponse>`

### `releaseEscrow`
Releases escrowed funds to the vendor.
- `caller`: (string) Authorized address.
- `paymentId`: (bigint) Escrow ID.
- `signer`: (Keypair) Optional.
- **Returns**: `Promise<Transaction | GetTransactionResponse>`

---

## Error Handling

The SDK throws `ZPayError` for network or contract-level failures.

```typescript
try {
    await kit.pay({ ... });
} catch (e) {
    if (e instanceof ZPayError) {
        console.error(`Error Code: ${e.code}, Msg: ${e.message}`);
        // Possible codes: "TIMEOUT", "FAILED", "PENDING", etc.
    }
}
```

---

## Data Types

### `MandateScope`
```typescript
interface MandateScope {
    ttl: number;
    transfer_limit?: bigint;
    contract_allowlist?: string[];
    function_allowlist?: string[];
}
```

### `DelegationPolicy`
```typescript
type DelegationPolicy = 
    | { type: "None" }
    | { type: "Full" }
    | { type: "Restricted", rules: { max_subdepth: number; budget_fraction?: number } };
```

---

## Professional Agent Example

```typescript
import { Keypair } from "@stellar/stellar-sdk";

const agentKey = Keypair.fromSecret("S...");

// 1. Identity Check
const isVerified = await kit.hasSoulID(userAddress);
if (!isVerified) throw new Error("User must have a SoulID to use this agent.");

// 2. Automated Payment (Build + Sign + Send + Poll)
const receipt = await kit.payEscrow({
    agent: agentKey.publicKey(),
    rootAnchor: userAddress,
    seller: vendorAddress,
    token: XLM_ID,
    amount: 100_0000000n,
    mandateId: 1n,
    sourceAccount: agentKey.publicKey(),
    signer: agentKey // One-step execution
});

console.log(`Payment confirmed in ledger! Hash: ${receipt.hash}`);
```
