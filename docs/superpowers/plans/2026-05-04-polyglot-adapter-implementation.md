# Polyglot Adapter & Complete Interoperability Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a polyglot cross-chain adapter on Stellar that supports EVM (ABI) and Cosmos/Solana (Borsh) serialization formats, and update verifiers on all networks.

**Architecture:** 
1. Update `Nexus` (Stellar) to allow specifying an `Ecosystem` for cross-chain transactions.
2. Refactor `AxelarAdapter` (Stellar) to encode payloads based on the target ecosystem (ABI vs Borsh).
3. Enhance Cosmos and Solana verifiers to handle the `REPUTATION` payload format.
4. Update integration tests to verify the full flow across all three ecosystems.

**Tech Stack:** Soroban (Rust), Solidity, CosmWasm (Rust), Anchor (Solana/Rust), Axelar GMP.

---

### Task 1: Update Core Types in Nexus (Stellar)

**Files:**
- Modify: `contracts/nexus/src/lib.rs`

- [ ] **Step 1: Define Ecosystem enum**
Add the `Ecosystem` enum to `contracts/nexus/src/lib.rs`.

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Ecosystem {
    Evm,
    Cosmos,
    Solana,
}
```

- [ ] **Step 2: Update CrossChainParams struct**
Update `CrossChainParams` to include the `ecosystem` field.

```rust
#[contracttype]
#[derive(Clone, Debug)]
pub struct CrossChainParams {
    pub destination_chain: soroban_sdk::String,
    pub destination_address: soroban_sdk::String,
    pub user_destination_address: soroban_sdk::Bytes,
    pub ecosystem: Ecosystem,
}
```

- [ ] **Step 3: Update `export_reputation` and `export_will_authority` calls**
Pass the ecosystem parameter to the adapter calls.

```rust
// Inside export_reputation
env.invoke_contract::<()>(
    &interop_config.adapter_address,
    &Symbol::new(&env, "send_reputation"),
    (
        _caller,
        cc.destination_chain,
        cc.destination_address,
        external_id,
        tier,
        cc.user_destination_address,
        nonce,
        token_type,
        cc.ecosystem, // Add this
    )
        .into_val(&env),
);

// Inside export_will_authority
env.invoke_contract::<()>(
    &interop_config.adapter_address,
    &Symbol::new(&env, "send_will_auth"),
    (
        _caller,
        cross_chain.destination_chain,
        cross_chain.destination_address,
        cross_chain.user_destination_address,
        auth.soul_id,
        auth.permissions,
        auth.expiry,
        cross_chain.ecosystem, // Add this
    )
        .into_val(&env),
);
```

- [ ] **Step 4: Verify build**
Run: `cargo build --target wasm32-unknown-unknown --release` in `contracts/nexus`
Expected: Compilation succeeds.

- [ ] **Step 5: Commit**
```bash
git add contracts/nexus/src/lib.rs
git commit -m "feat(nexus): add ecosystem support to cross-chain params"
```

---

### Task 2: Implement Polyglot Encoding in AxelarAdapter (Stellar)

**Files:**
- Modify: `contracts/adapters/axelar/src/lib.rs`

- [ ] **Step 1: Define Ecosystem enum in Adapter**
Match the enum defined in Nexus.

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Ecosystem {
    Evm,
    Cosmos,
    Solana,
}
```

- [ ] **Step 2: Update function signatures**
Update `send_reputation` and `send_will_auth` to accept `ecosystem`.

```rust
pub fn send_reputation(
    env: Env,
    caller: Address,
    destination_chain: String,
    destination_address: String,
    external_id: String,
    tier: u32,
    user_evm_address: Bytes,
    nonce: u64,
    token_type: Symbol,
    ecosystem: Ecosystem, // Add this
) -> Result<(), Error>

pub fn send_will_auth(
    env: Env,
    caller: Address,
    destination_chain: String,
    destination_address: String,
    will_evm_address: Bytes,
    soul_id: u32,
    permissions: u64,
    expiry: u64,
    ecosystem: Ecosystem, // Add this
) -> Result<(), Error>
```

- [ ] **Step 3: Implement Borsh encoding for Reputation**
Add `encode_reputation_payload_borsh` (Little-Endian, no padding).

```rust
fn encode_reputation_payload_borsh(env: &Env, external_id: &String, tier: u32, user: &Bytes, nonce: u64, token_type: Symbol) -> Bytes {
    let mut payload = Bytes::new(env);
    payload.append(&Bytes::from_array(env, &[1u8])); // Type 1: Reputation
    
    // Borsh: external_id (as hash), tier (u32), user (32 bytes), nonce (u64), token_type (as hash)
    payload.append(&env.crypto().keccak256(&external_id.clone().to_xdr(env)).into());
    
    let t_le = tier.to_le_bytes();
    payload.append(&Bytes::from_array(env, &t_le));

    let mut user_bytes = [0u8; 32];
    user.copy_into_slice(&mut user_bytes[12..32]); // Assuming 20 byte EVM address for now, or full 32 for others
    payload.append(&Bytes::from_array(env, &user_bytes));

    let n_le = nonce.to_le_bytes();
    payload.append(&Bytes::from_array(env, &n_le));

    payload.append(&env.crypto().keccak256(&token_type.to_xdr(env)).into());
    
    payload
}
```

- [ ] **Step 4: Update `send_reputation` logic to switch encoding**
```rust
let payload = match ecosystem {
    Ecosystem::Evm => Self::encode_reputation_payload(env, &external_id, tier as u8, &user_evm_address, nonce, token_type),
    _ => Self::encode_reputation_payload_borsh(env, &external_id, tier, &user_evm_address, nonce, token_type),
};
```

- [ ] **Step 5: Commit**
```bash
git add contracts/adapters/axelar/src/lib.rs
git commit -m "feat(axelar-adapter): implement polyglot encoding for reputation"
```

---

### Task 3: Update Cosmos Verifier (CosmWasm)

**Files:**
- Modify: `verifiers/cosmos/src/contract.rs`
- Modify: `verifiers/cosmos/src/state.rs`
- Modify: `verifiers/cosmos/src/msg.rs`

- [ ] **Step 1: Add Reputation state**
In `verifiers/cosmos/src/state.rs`, add `REPUTATIONS`.

```rust
pub const REPUTATIONS: Map<(&str, &str), Reputation> = Map::new("reputations"); // (user_addr, token_type_hash)

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Reputation {
    pub external_id: String,
    pub tier: u32,
    pub nonce: u64,
}
```

- [ ] **Step 2: Implement Reputation handler in `execute_axelar_message`**
Handle `payload[0] == 1`.

```rust
if payload_bytes[0] == 1 {
    let mut data = &payload_bytes[1..];
    let external_id_hash = <[u8; 32]>::deserialize(&mut data).map_err(|_| ContractError::InvalidPayload {})?;
    let tier = u32::deserialize(&mut data).map_err(|_| ContractError::InvalidPayload {})?;
    let user_bytes = <[u8; 32]>::deserialize(&mut data).map_err(|_| ContractError::InvalidPayload {})?;
    let nonce = u64::deserialize(&mut data).map_err(|_| ContractError::InvalidPayload {})?;
    let token_type_hash = <[u8; 32]>::deserialize(&mut data).map_err(|_| ContractError::InvalidPayload {})?;

    let user_addr = hex::encode(user_bytes);
    let token_type = hex::encode(token_type_hash);
    let external_id = hex::encode(external_id_hash);

    let rep = Reputation { external_id, tier, nonce };
    REPUTATIONS.save(deps.storage, (&user_addr, &token_type), &rep)?;

    return Ok(Response::new().add_attribute("action", "update_reputation"));
}
```

- [ ] **Step 3: Commit**
```bash
git add verifiers/cosmos/src/
git commit -m "feat(cosmos-verifier): add reputation support"
```

---

### Task 4: Update Solana Verifier (Anchor)

**Files:**
- Modify: `verifiers/solana/programs/zolvency-verifier-solana/src/lib.rs`

- [ ] **Step 1: Add Reputation Account and PDA**
Define `Reputation` account and update `Execute` context.

```rust
#[account]
pub struct Reputation {
    pub external_id: [u8; 32],
    pub tier: u32,
    pub nonce: u64,
}

// In Execute struct, add reputation PDA
#[account(
    init_if_needed,
    payer = payer,
    space = 8 + 32 + 4 + 8,
    seeds = [b"reputation", payload[1..33].as_ref(), payload[77..109].as_ref()], // user + token_type
    bump
)]
pub reputation: Account<'info, Reputation>,
```

- [ ] **Step 2: Implement Reputation handler in `execute`**
Handle `payload[0] == 1`.

```rust
if payload[0] == 1 {
    let mut data = &payload[1..];
    let external_id = <[u8; 32]>::deserialize(&mut data)?;
    let tier = u32::deserialize(&mut data)?;
    let user = <[u8; 32]>::deserialize(&mut data)?;
    let nonce = u64::deserialize(&mut data)?;
    let token_type = <[u8; 32]>::deserialize(&mut data)?;

    let rep = &mut ctx.accounts.reputation;
    require!(nonce > rep.nonce, ErrorCode::InvalidNonce);
    rep.external_id = external_id;
    rep.tier = tier;
    rep.nonce = nonce;
}
```

- [ ] **Step 3: Commit**
```bash
git add verifiers/solana/
git commit -m "feat(solana-verifier): add reputation support"
```

---

### Task 5: Final Verification & Test Script

**Files:**
- Create: `scripts/test_polyglot_flow.sh`

- [ ] **Step 1: Write test script**
Create a script that triggers the `mint` on Stellar with different `ecosystem` targets and logs the output.

- [ ] **Step 2: Run full integration test**
Run the script and verify that Axelar logs show different payload sizes for EVM vs Borsh.

- [ ] **Step 3: Commit**
```bash
git add scripts/test_polyglot_flow.sh
git commit -m "test: add polyglot cross-chain integration test"
```
