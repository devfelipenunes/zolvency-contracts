# GitHub Identity Contract

A soulbound identity token on [Stellar/Soroban](https://soroban.stellar.org) that verifies GitHub developer activity on-chain. Each address can hold exactly one token, permanently tied to their GitHub profile and contribution history.

---

## Table of Contents

- [Overview](#overview)
- [Tier System](#tier-system)
- [Architecture](#architecture)
- [Getting Started](#getting-started)
- [Contract API](#contract-api)
- [Security Model](#security-model)
- [Storage Design](#storage-design)
- [Testing](#testing)
- [Roadmap](#roadmap)

---

## Overview

GitHub Identity issues non-transferable (soulbound) NFTs that encode a developer's GitHub contribution history. Tokens are minted with a server-side ECDSA signature and zkTLS proof, ensuring the underlying GitHub data is authentic and tamper-resistant.

**Key properties:**

- One token per address — enforced on-chain
- Tier upgrades via `update_token` as contributions grow
- On-chain SVG metadata, no external dependency
- Nonce-based replay protection on every mint

---

## Tier System

Tier is calculated directly from total GitHub contributions at mint or update time.

| # | Tier | Contributions | Color |
|---|------|--------------|-------|
| 1 | Novice | 0 – 199 | Bronze `#CD7F32` |
| 2 | Pro | 200 – 999 | Silver `#C0C0C0` |
| 3 | Architect | 1 000 – 2 999 | Gold `#FFD700` |
| 4 | Legend | 3 000 – 4 999 | Platinum `#E5E4E2` |
| 5 | Singularity | 5 000+ | Neon Green `#39FF14` |

---

## Architecture

```
github-identity/
├── src/
│   ├── lib.rs       # Public contract interface & entry points
│   ├── types.rs     # Domain types (GithubData, Tier, Config, Error)
│   ├── storage.rs   # All storage reads and writes
│   └── test.rs      # Unit tests
└── Cargo.toml
```

The three modules have strict separation of concerns:

- **`types.rs`** — pure Rust, no storage calls. Safe to test without `Env`.
- **`storage.rs`** — all `env.storage()` calls live here and nowhere else. Easy to audit for storage layout.
- **`lib.rs`** — orchestrates the two above. No business logic, no raw storage.

---

## Getting Started

### Prerequisites

- Rust `1.74+` with `wasm32-unknown-unknown` target
- [Stellar CLI](https://github.com/stellar/stellar-cli)

```bash
rustup target add wasm32-unknown-unknown
cargo install stellar-cli
```

### Build

```bash
stellar contract build
```

The compiled WASM lands at:
```
target/wasm32-unknown-unknown/release/github_identity.wasm
```

### Test

```bash
cargo test
# with stdout
cargo test -- --nocapture
# single test
cargo test test_mint_returns_token_id_one
```

### Deploy

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/github_identity.wasm \
  --network testnet \
  --source <YOUR_SECRET_KEY>
```

### Initialize

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  --source <ADMIN_SECRET> \
  -- initialize \
  --admin <ADMIN_ADDRESS> \
  --access_control <ACCESS_CONTROL_ADDRESS> \
  --treasury <TREASURY_ADDRESS> \
  --mint_fee 1000000
```

---

## Contract API

### Write functions

#### `initialize`
Sets up the contract. Can only be called once.

| Param | Type | Description |
|-------|------|-------------|
| `admin` | `Address` | Admin for configuration changes |
| `access_control` | `Address` | Access control contract |
| `treasury` | `Address` | Fee recipient |
| `mint_fee` | `i128` | Mint fee in stroops (0 = free) |

---

#### `mint`
Issues a new identity token to the caller.

| Param | Type | Description |
|-------|------|-------------|
| `caller` | `Address` | Address receiving the token |
| `signature` | `BytesN<64>` | ECDSA signature from authorized server |
| `username` | `String` | GitHub username |
| `contributions` | `u32` | Total GitHub contributions |
| `proof_data` | `Bytes` | zkTLS proof |
| `referrer` | `Option<Address>` | Optional referrer (future revenue split) |
| `nonce` | `u64` | Must match `get_nonce(caller)` |

Returns the new `token_id: u64`.

---

#### `update_token`
Updates contribution data for an existing token. Caller must own the token.

| Param | Type | Description |
|-------|------|-------------|
| `caller` | `Address` | Token owner |
| `token_id` | `u64` | Token to update |
| `username` | `String` | Updated username |
| `contributions` | `u32` | Updated contribution count |
| `proof_data` | `Bytes` | Fresh zkTLS proof |

---

### Read functions

| Function | Returns | Description |
|----------|---------|-------------|
| `get_token_data(token_id)` | `GithubData` | Full token data |
| `get_user_token(user)` | `u64` | Token ID for an address |
| `has_identity(user)` | `bool` | Whether address holds a token |
| `get_nonce(user)` | `u64` | Next valid mint nonce |
| `get_mint_fee()` | `i128` | Current fee in stroops |
| `get_token_svg(token_id)` | `String` | On-chain SVG image |
| `list_tokens_of_user(user)` | `Vec<u64>` | Token IDs (max 1 — soulbound) |

---

### Admin functions

All admin functions require `admin.require_auth()` and verify the caller matches the stored admin address.

| Function | Description |
|----------|-------------|
| `set_mint_fee(admin, new_fee)` | Update the mint fee |
| `set_access_control(admin, address)` | Update access control contract |
| `set_treasury(admin, address)` | Update treasury contract |

---

## Security Model

### What is implemented

| Mechanism | Status |
|-----------|--------|
| Soulbound (non-transferable) | ✅ Enforced |
| Nonce-based replay protection | ✅ Active |
| One token per address | ✅ Enforced |
| Admin access control | ✅ Active |
| Duplicate mint guard | ✅ Active |

### What is pending

| Mechanism | Status |
|-----------|--------|
| ECDSA signature verification | ⚠️ Stub — must implement before mainnet |
| Mint fee payment transfer | ⚠️ Stub — contract reverts if fee > 0 |
| zkTLS proof validation | ⚠️ Off-chain only currently |

> **Do not deploy to mainnet without implementing ECDSA verification.**  
> The `signature` parameter is currently accepted but not checked.

---

## Storage Design

### Persistent storage (no expiry)

| Key | Value | Description |
|-----|-------|-------------|
| `"CONFIG"` | `Config` | Admin, treasury, access control, fee |
| `"TOKEN_CTR"` | `u64` | Auto-increment token counter |
| `("TOK", token_id)` | `GithubData` | Token data by ID |
| `("HLD", address)` | `u64` | Token ID by holder address |
| `("HAS", address)` | `bool` | Identity existence flag |

### Temporary storage (30-day TTL)

| Key | Value | Description |
|-----|-------|-------------|
| `("NON", address)` | `u64` | Replay-protection nonce |

Nonce TTL is refreshed on every mint. An address that hasn't minted in 30 days resets to nonce `0`.

---

## Testing

The test suite covers all public entry points and error paths.

### Test categories

| Category | What is covered |
|----------|----------------|
| Initialization | Happy path, double-init rejection |
| Minting | Token ID sequence, identity flag, empty username, duplicate mint, wrong nonce |
| Nonce | Initial value, increment after mint |
| Token queries | Data correctness, missing token |
| Update | Contribution and tier change, non-owner rejection, missing identity |
| Tier calculation | All boundaries including `u32::MAX` |
| SVG generation | All 5 tiers, exact output for Architect, missing token |
| Admin | Fee update, access control, treasury — happy path and non-admin rejection |

```bash
cargo test
```

---

## Roadmap

- [ ] ECDSA server signature verification
- [ ] Native XLM mint fee payment via token client
- [ ] zkTLS proof validation (on-chain or verifier contract)
- [ ] Referrer revenue split
- [ ] Persistent storage TTL management
- [ ] Frontend SDK