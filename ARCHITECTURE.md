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
    fn get_owner_passkey(env: Env, token_id: u64) -> BytesN<32>; // Passkey binding
}
```

### Key Innovations
1.  **Passkey Binding**: Tokens are bound to a WebAuthn/Passkey (secp256r1) rather than just a private key, ensuring hardware-level security.
2.  **Business TTL**: Tokens have a configurable expiration period (e.g., 90 days for GitHub) to ensure reputation is current.
3.  **Sybil Resistance**: Each token maps a unique external identifier (e.g., GitHub User ID hash) to a `token_id`. If a user attempts to mint a second token with the same external ID, the protocol invalidates the previous token.

---

## 3. Implementation Status
| Component | Status | Verification |
|-----------|--------|--------------|
| **Zolvency Registry** | ✅ Functional | Integration tests passing |
| **GitHub Identity** | ✅ Refactored | v6.0 Interface implemented |
| **Sybil Resistance** | ✅ Active | Validated via `test_sybil_resistance_mapping` |
| **Passkey Binding** | ✅ Implemented | Validated via `test_mint_with_passkey_and_expiry` |
| **Cross-Contract Discovery** | ✅ Functional | Validated via `test_registry_integration_with_github_token` |

---

## 4. Roadmap
- **P1-03 (Bank-SBT)**: Implementation of zk-email DKIM circuits for Brazilian banks.
- **P1-05 (Social Recovery)**: Sovereign recovery flow using bank email DKIM as a root of trust.
- **RE-01 (Zolvency SDK)**: TypeScript library to consume `get_user_reputation` and provide LTV (Loan-to-Value) multipliers to lending protocols.
