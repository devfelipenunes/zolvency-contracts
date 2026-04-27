# Multi-Protocol Interoperability Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable switching between Axelar and LayerZero V2 for cross-chain reputation pushes on Stellar.

**Architecture:** Use the Adapter Pattern to decouple the core Identity contract from specific bridge protocols.

**Tech Stack:** Soroban (Rust), Solidity, Axelar GMP, LayerZero V2.

---

### Task 1: [Stellar] Extend Types and Storage
Add support for selecting between different interoperability protocols.

**Files:**
- Modify: `packages/stellar/contracts/github-identity/src/types.rs`
- Modify: `packages/stellar/contracts/github-identity/src/storage.rs`

- [ ] **Step 1: Update types.rs with InteropProtocol enum**
```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InteropProtocol {
    None,
    Axelar,
    LayerZero,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct InteropConfig {
    pub active_protocol: InteropProtocol,
    pub adapter_address: Address,
}
```

- [ ] **Step 2: Add InteropConfig to DataKey in types.rs**
```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    AxelarConfig,
    InteropConfig, // Add this
    // ... other keys
}
```

- [ ] **Step 3: Add storage helpers in storage.rs**
```rust
pub fn get_interop_config(env: &Env) -> Result<InteropConfig, Error> {
    env.storage().instance().get(&DataKey::InteropConfig).ok_or(Error::NotInitialized)
}

pub fn set_interop_config(env: &Env, config: &InteropConfig) {
    env.storage().instance().set(&DataKey::InteropConfig, config);
}
```

- [ ] **Step 4: Commit**
```bash
git add packages/stellar/contracts/github-identity/src/types.rs packages/stellar/contracts/github-identity/src/storage.rs
git commit -m "feat(stellar): add multi-protocol interop types and storage"
```

---

### Task 2: [Stellar] Define Interop Interface
Create a common trait that all adapters must implement.

**Files:**
- Create: `packages/stellar/contracts/github-identity/src/interop.rs`

- [ ] **Step 1: Define the Messenger trait**
```rust
use soroban_sdk::{Address, Bytes, Env, String};

pub trait MessengerTrait {
    fn send_reputation(
        env: Env,
        caller: Address,
        destination_chain: String,
        destination_address: String,
        payload: Bytes,
    ) -> Result<(), crate::types::Error>;
}
```

- [ ] **Step 2: Commit**
```bash
git add packages/stellar/contracts/github-identity/src/interop.rs
git commit -m "feat(stellar): define common messenger trait"
```

---

### Task 3: [Stellar] Refactor Axelar into Adapter
Move the existing Axelar logic into its own dedicated adapter file.

**Files:**
- Create: `packages/stellar/contracts/github-identity/src/axelar_adapter.rs`
- Modify: `packages/stellar/contracts/github-identity/src/lib.rs` (to register the module)

- [ ] **Step 1: Implement AxelarAdapter**
Extract the logic from `lib.rs` that calls `axelar_client.pay_gas` and `axelar_client.call_contract`.

- [ ] **Step 2: Commit**
```bash
git add packages/stellar/contracts/github-identity/src/axelar_adapter.rs
git commit -m "refactor(stellar): move axelar logic to adapter"
```

---

### Task 4: [Stellar] Update Identity Contract Logic
Modify `mint` and `update_token` to use the active adapter.

**Files:**
- Modify: `packages/stellar/contracts/github-identity/src/lib.rs`

- [ ] **Step 1: Implement dispatch logic**
```rust
let interop_config = storage::get_interop_config(&env)?;
match interop_config.active_protocol {
    InteropProtocol::Axelar => {
        // Call AxelarAdapter
    },
    InteropProtocol::LayerZero => {
        // Call LayerZeroAdapter (Task 5)
    },
    InteropProtocol::None => {}
}
```

- [ ] **Step 2: Add admin function to switch protocol**
```rust
pub fn set_active_protocol(env: Env, admin: Address, protocol: InteropProtocol, adapter: Address) -> Result<(), Error> {
    admin.require_auth();
    // Verify admin
    storage::set_interop_config(&env, &InteropConfig { active_protocol: protocol, adapter_address: adapter });
    Ok(())
}
```

- [ ] **Step 3: Commit**
```bash
git add packages/stellar/contracts/github-identity/src/lib.rs
git commit -m "feat(stellar): implement protocol switching in core identity"
```

---

### Task 5: [EVM] Create Dual Verifiers
Deploy separate verifier contracts to compare performance.

**Files:**
- Create: `packages/evm/src/ZolvencyVerifierAxelar.sol`
- Create: `packages/evm/src/ZolvencyVerifierLayerZero.sol`

- [ ] **Step 1: Implement Axelar Verifier (Current logic)**
- [ ] **Step 2: Implement LayerZero OApp Verifier**
- [ ] **Step 3: Commit**
```bash
git add packages/evm/src/ZolvencyVerifier*.sol
git commit -m "feat(evm): add dual verifiers for axelar and layerzero"
```
