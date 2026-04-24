# Zolvency Protocol: Technical Architecture v6.0

## Overview
Zolvency is the Credit Layer of Stellar, providing a decentralized infrastructure for Soulbound Tokens (SBTs) that represent verifiable reputation (GitHub activity, Bank income, On-chain history).

The architecture follows a **Hub & Spoke** model, ensuring extensibility and a unified interface for DeFi protocols.

---

## 1. The Hub: Zolvency Registry
The Registry acts as the single source of truth for the Zolvency ecosystem.

- **Token Discovery**: A central repository of all officially recognized SBT contracts.
- **Unified Reputation API**: `get_user_reputation(address)` returns a map of all active tokens held by a user across the entire protocol.
- **Security Orchestration**: Stores the `authorized_signer` public key, used by Spoke contracts to verify ZK proofs and server attestations.

---

## 2. The Spoke: Zolvency Tokens (SBTs)
All tokens (GitHub-SBT, Bank-SBT, Activity-SBT) implement the `ZolvencyTokenTrait`.

### Standard Interface
```rust
pub trait ZolvencyTokenTrait {
    fn get_token_type(env: Env) -> Symbol;      // e.g., "github", "bank"
    fn get_source(env: Env) -> String;         // e.g., "zk-email-dkim"
    fn is_valid(env: Env, token_id: u64) -> bool; // Business TTL check
    fn get_expiry(env: Env, token_id: u64) -> u64; // UNIX Timestamp
    fn get_owner_passkey(env: Env, token_id: u64) -> Option<BytesN<65>>; // Optional Passkey binding
}
```

### Key Innovations
1.  **Optional Passkey Binding**: Tokens can be bound to a WebAuthn/Passkey (secp256r1) for hardware-level security. This is optional and provides "Opt-in security".
2.  **Business TTL**: Tokens have a configurable expiration period (e.g., 90 days for GitHub) to ensure reputation is current.
3.  **Sybil Resistance**: Each token maps a unique external identifier (e.g., GitHub User ID hash) to a `token_id`. If a user attempts to mint a second token with the same external ID, the protocol invalidates the previous token.

---

## 3. Modular Interoperability
The protocol supports cross-chain reputation export via the **Adapter Pattern**.

- **GithubIdentityContract (Spoke)**: Processes and stores reputation on Stellar.
- **Interoperability Adapters**: Independent contracts that handle the communication with other chains.
    - `AxelarAdapter`: Uses Axelar GMP for automatic Push updates to EVM.
    - `AuthorityPullAdapter`: Emits verifiable events for off-chain signatures (Pull model).

### Cross-chain Workflow
1. User calls `mint` or `update_token` on Stellar.
2. If `cross_chain` params are provided, Identity calls the active **Adapter**.
3. Adapter dispatches the message (via Axelar Gateway or Events).
4. Destination contract (EVM) verifies and updates the user's local status.

---

## 4. Implementation Status
| Component | Status | Verification |
|-----------|--------|--------------|
| **Zolvency Registry** | ✅ Functional | Integration tests passing |
| **GitHub Identity** | ✅ Modular | v7.0 Modular Interface |
| **Axelar Adapter** | ✅ Functional | Validated on Sepolia Testnet |
| **Authority-Pull** | ✅ Implemented | Unit tests passing |
| **Sybil Resistance** | ✅ Active | Validated via tests |
| **Passkey Binding** | ✅ Optional | Opt-in hardware security |

---

## 5. Roadmap
- **P1-03 (Bank-SBT)**: Implementation of zk-email DKIM circuits for Brazilian banks.
- **RE-01 (Zolvency SDK)**: TypeScript library to consume `get_user_reputation` and provide LTV (Loan-to-Value) multipliers to lending protocols.

