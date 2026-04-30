# SoulID (u32) Test Update Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Update tests in `github-identity` and `zolvency-registry` to use `soul_id: u32` instead of `Address`.

**Architecture:** Switch identity linking from `Address` to `u32` Soul IDs across all test suites, including mock contracts and contract client calls.

**Tech Stack:** Rust, Soroban SDK

---

### Task 1: Update `github-identity` Tests

**Files:**
- Modify: `packages/stellar/contracts/github-identity/src/test.rs`

- [ ] **Step 1: Update `MockSoul` contract**
Update `MockSoul` to use `u32` Soul IDs and provide a `get_soul` method.

- [ ] **Step 2: Update `mint_for` helper**
Update signature and implementation to use `soul_id: u32`.

- [ ] **Step 3: Update all test cases**
Replace `Address::generate(&env)` for users with `u32` values and update method calls.

- [ ] **Step 4: Verify `github-identity` tests**
Run: `cargo test -p github-identity`
Expected: PASS

### Task 2: Update `zolvency-registry` Tests

**Files:**
- Modify: `packages/stellar/contracts/zolvency-registry/src/test.rs`

- [ ] **Step 1: Update `MockSoul` contract**
Ensure `MockSoul` matches the one in `github-identity`.

- [ ] **Step 2: Update all test cases**
Update `get_soul_reputation`, `lock_soul_reputation`, `is_soul_locked`, `apply_soul_slashing`, `is_soul_blacklisted` calls.

- [ ] **Step 3: Verify `zolvency-registry` tests**
Run: `cargo test -p zolvency-registry`
Expected: PASS

### Task 3: Final Verification

- [ ] **Step 1: Run all tests in the workspace**
Run: `cargo test`
Expected: PASS
