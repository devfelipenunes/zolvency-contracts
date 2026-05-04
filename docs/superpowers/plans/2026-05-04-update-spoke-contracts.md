# Update Spoke Contracts for Cross-Chain Compatibility Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Update all Spoke contracts (`github`, `flow`, `gig`, `stamp`, `soul`, `will`) to use the new `CrossChainParams` struct and `Ecosystem` enum, ensuring compatibility with the `Nexus` contract.

**Architecture:** 
1. Define the `Ecosystem` enum in each contract's `types.rs` (or `lib.rs` if `types.rs` is missing).
2. Update the `CrossChainParams` struct to include the `ecosystem` field.
3. Update any `mint` or `update` functions that pass `CrossChainParams` to `Nexus::export_reputation` to handle the new field.

**Tech Stack:** Rust, Soroban SDK

---

### Task 1: Update GitHub Spoke Contract

**Files:**
- Modify: `contracts/github/src/types.rs`
- Modify: `contracts/github/src/lib.rs`
- Test: `contracts/github/src/test.rs` (if exists and uses `CrossChainParams`)

- [ ] **Step 1: Add Ecosystem enum and update CrossChainParams in `types.rs`**

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Ecosystem {
    Evm,
    Cosmos,
    Solana,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct CrossChainParams {
    pub destination_chain: String,
    pub destination_address: String,
    pub user_destination_address: Bytes,
    pub ecosystem: Ecosystem,
}
```

- [ ] **Step 2: Verify `lib.rs` doesn't need changes to logic (it just passes the struct through)**

- [ ] **Step 3: Update tests in `contracts/github/src/test.rs` to provide the `ecosystem` field**

### Task 2: Update Flow Spoke Contract

**Files:**
- Modify: `contracts/flow/src/types.rs`
- Modify: `contracts/flow/src/lib.rs`
- Test: `contracts/flow/src/test.rs`

- [ ] **Step 1: Add Ecosystem enum and update CrossChainParams in `types.rs`**

- [ ] **Step 2: Update tests in `contracts/flow/src/test.rs`**

### Task 3: Update Gig Spoke Contract

**Files:**
- Modify: `contracts/gig/src/types.rs`
- Modify: `contracts/gig/src/lib.rs`
- Test: `contracts/gig/src/test.rs`

- [ ] **Step 1: Add Ecosystem enum and update CrossChainParams in `types.rs`**

- [ ] **Step 2: Update tests in `contracts/gig/src/test.rs`**

### Task 4: Update Stamp Spoke Contract

**Files:**
- Modify: `contracts/stamp/src/types.rs`
- Modify: `contracts/stamp/src/lib.rs`
- Test: `contracts/stamp/src/test.rs`

- [ ] **Step 1: Add Ecosystem enum and update CrossChainParams in `types.rs`**

- [ ] **Step 2: Update tests in `contracts/stamp/src/test.rs`**

### Task 5: Add Compatibility Types to Soul Spoke Contract (Optional/For Consistency)

**Files:**
- Modify: `contracts/soul/src/types.rs`

- [ ] **Step 1: Add Ecosystem enum and CrossChainParams in `types.rs`**

### Task 6: Add Compatibility Types to Will Spoke Contract (Optional/For Consistency)

**Files:**
- Modify: `contracts/will/src/lib.rs`

- [ ] **Step 1: Add Ecosystem enum and CrossChainParams in `lib.rs`**

### Task 7: Verification

- [ ] **Step 1: Compile all contracts to ensure no regressions**
