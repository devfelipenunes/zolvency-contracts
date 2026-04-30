# Sovereign Soul Token Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Re-architect the `zolvency-soul` contract to use a Passkey-First identity system with cryptographic recovery.

**Architecture:** Use `SoulID` (u32) as the primary internal identifier. Map Passkey public keys to `SoulID`. Implement recovery using a secondary "Recovery Public Key" and signature verification.

**Tech Stack:** Rust, Soroban SDK.

---

### Task 1: Setup New Project Structure

**Files:**
- Create: `packages/stellar/contracts/zolvency-soul/src/types.rs`
- Create: `packages/stellar/contracts/zolvency-soul/src/storage.rs`
- Modify: `packages/stellar/contracts/zolvency-soul/src/lib.rs`

- [ ] **Step 1: Create `types.rs` with new structs**

```rust
use soroban_sdk::{contracterror, contracttype, BytesN};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotAuthorized = 2,
    SoulAlreadyExists = 3,
    NotInitialized = 4,
    CounterOverflow = 5,
    SoulNotFound = 6,
    InvalidRecoverySignature = 7,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Relayer,
    TotalSouls,
    SoulById(u32),                 // SoulID -> SoulData
    SoulByPasskey(BytesN<65>),     // Passkey PubKey -> SoulID
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SoulData {
    pub id: u32,
    pub passkey: BytesN<65>,           // secp256r1 pubkey
    pub recovery_pubkey: BytesN<65>,   // secp256r1 pubkey for recovery
    pub minted_at: u64,
}
```

- [ ] **Step 2: Create `storage.rs` for modularity**

```rust
use soroban_sdk::{Env, Address, BytesN};
use crate::types::{DataKey, SoulData, Error};

const DAY_IN_LEDGERS: u32 = 17_280;
const ONE_YEAR: u32 = 365 * DAY_IN_LEDGERS;

pub fn extend_instance(env: &Env) {
    env.storage().instance().extend_ttl(ONE_YEAR, ONE_YEAR);
}

pub fn extend_persistent(env: &Env, key: &DataKey) {
    env.storage().persistent().extend_ttl(key, ONE_YEAR, ONE_YEAR);
}

pub fn get_admin(env: &Env) -> Result<Address, Error> {
    env.storage().instance().get(&DataKey::Admin).ok_or(Error::NotInitialized)
}

pub fn get_relayer(env: &Env) -> Result<Address, Error> {
    env.storage().instance().get(&DataKey::Relayer).ok_or(Error::NotInitialized)
}

pub fn get_total_souls(env: &Env) -> u32 {
    env.storage().instance().get(&DataKey::TotalSouls).unwrap_or(0)
}

pub fn increment_total_souls(env: &Env) -> u32 {
    let total = get_total_souls(env) + 1;
    env.storage().instance().set(&DataKey::TotalSouls, &total);
    total
}

pub fn set_soul(env: &Env, soul: &SoulData) {
    let id_key = DataKey::SoulById(soul.id);
    let pk_key = DataKey::SoulByPasskey(soul.passkey.clone());
    
    env.storage().persistent().set(&id_key, soul);
    env.storage().persistent().set(&pk_key, &soul.id);
    
    extend_persistent(env, &id_key);
    extend_persistent(env, &pk_key);
}

pub fn get_soul_id_by_passkey(env: &Env, passkey: &BytesN<65>) -> Option<u32> {
    env.storage().persistent().get(&DataKey::SoulByPasskey(passkey.clone()))
}

pub fn get_soul_by_id(env: &Env, id: u32) -> Option<SoulData> {
    env.storage().persistent().get(&DataKey::SoulById(id))
}

pub fn remove_passkey_mapping(env: &Env, passkey: &BytesN<65>) {
    env.storage().persistent().remove(&DataKey::SoulByPasskey(passkey.clone()));
}
```

- [ ] **Step 3: Update `lib.rs` modules**

```rust
#![no_std]

mod types;
mod storage;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, symbol_short, Address, Bytes, BytesN, Env, Vec};
pub use types::{Error, SoulData};

#[contract]
pub struct ZolvencySoulContract;
```

- [ ] **Step 4: Commit structure**

```bash
git add packages/stellar/contracts/zolvency-soul/src/types.rs packages/stellar/contracts/zolvency-soul/src/storage.rs packages/stellar/contracts/zolvency-soul/src/lib.rs
git commit -m "refactor: setup modular structure for sovereign soul"
```

---

### Task 2: Implement Initialization and Mint

**Files:**
- Modify: `packages/stellar/contracts/zolvency-soul/src/lib.rs`

- [ ] **Step 1: Implement `initialize`**

```rust
#[contractimpl]
impl ZolvencySoulContract {
    pub fn initialize(env: Env, admin: Address, relayer: Address) -> Result<(), Error> {
        if env.storage().instance().has(&types::DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&types::DataKey::Admin, &admin);
        env.storage().instance().set(&types::DataKey::Relayer, &relayer);
        env.storage().instance().set(&types::DataKey::TotalSouls, &0u32);
        storage::extend_instance(&env);
        Ok(())
    }
}
```

- [ ] **Step 2: Implement `mint`**

```rust
    pub fn mint(
        env: Env,
        relayer: Address,
        passkey: BytesN<65>,
        recovery_pubkey: BytesN<65>,
    ) -> Result<u32, Error> {
        relayer.require_auth();
        
        let stored_relayer = storage::get_relayer(&env)?;
        if relayer != stored_relayer {
            return Err(Error::NotAuthorized);
        }

        if storage::get_soul_id_by_passkey(&env, &passkey).is_some() {
            return Err(Error::SoulAlreadyExists);
        }

        let soul_id = storage::increment_total_souls(&env);
        let soul_data = SoulData {
            id: soul_id,
            passkey: passkey.clone(),
            recovery_pubkey,
            minted_at: env.ledger().timestamp(),
        };

        storage::set_soul(&env, &soul_data);

        env.events().publish(
            (symbol_short!("soul"), symbol_short!("minted"), soul_id),
            passkey,
        );

        Ok(soul_id)
    }
```

- [ ] **Step 3: Commit Mint logic**

```bash
git add packages/stellar/contracts/zolvency-soul/src/lib.rs
git commit -m "feat: implement sovereign soul minting"
```

---

### Task 3: Implement Discovery and Recovery

**Files:**
- Modify: `packages/stellar/contracts/zolvency-soul/src/lib.rs`

- [ ] **Step 1: Implement `get_soul_by_passkey`**

```rust
    pub fn get_soul_by_passkey(env: Env, passkey: BytesN<65>) -> Option<SoulData> {
        let id = storage::get_soul_id_by_passkey(&env, &passkey)?;
        storage::get_soul_by_id(&env, id)
    }
```

- [ ] **Step 2: Implement `recover_soul`**

```rust
    pub fn recover_soul(
        env: Env,
        relayer: Address,
        old_passkey: BytesN<65>,
        new_passkey: BytesN<65>,
        recovery_signature: BytesN<64>,
    ) -> Result<(), Error> {
        relayer.require_auth();
        
        let stored_relayer = storage::get_relayer(&env)?;
        if relayer != stored_relayer {
            return Err(Error::NotAuthorized);
        }

        let soul_id = storage::get_soul_id_by_passkey(&env, &old_passkey).ok_or(Error::SoulNotFound)?;
        let mut soul_data = storage::get_soul_by_id(&env, soul_id).unwrap();

        // Verify recovery signature: sign(hash(old_passkey + new_passkey))
        let mut msg = Vec::new(&env);
        msg.append(&old_passkey.clone().into());
        msg.append(&new_passkey.clone().into());
        let msg_hash = env.crypto().sha256(&msg);

        // This is the core sovereign recovery check
        env.crypto().secp256r1_verify(
            &soul_data.recovery_pubkey,
            &msg_hash,
            &recovery_signature
        );

        // Update mappings
        storage::remove_passkey_mapping(&env, &old_passkey);
        soul_data.passkey = new_passkey.clone();
        storage::set_soul(&env, &soul_data);

        env.events().publish(
            (symbol_short!("soul"), symbol_short!("recovered"), soul_id),
            new_passkey,
        );

        Ok(())
    }
```

- [ ] **Step 3: Commit Recovery logic**

```bash
git add packages/stellar/contracts/zolvency-soul/src/lib.rs
git commit -m "feat: implement sovereign soul recovery"
```

---

### Task 4: Update Tests and Verify

**Files:**
- Modify: `packages/stellar/contracts/zolvency-soul/src/test.rs`

- [ ] **Step 1: Rewrite tests for the new architecture**

(Test code will use `BytesN<65>` and simulate the recovery signature)

- [ ] **Step 2: Run tests**

Run: `cargo test -p zolvency-soul`
Expected: PASS

- [ ] **Step 3: Final Commit**

```bash
git add packages/stellar/contracts/zolvency-soul/src/test.rs
git commit -m "test: verify sovereign soul mint and recovery"
```
