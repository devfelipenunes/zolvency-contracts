# Fix Axelar Interoperability Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable cross-chain reputation updates by fixing AxelarAdapter gas payment, enabling unit tests, and correcting deployment scripts.

**Architecture:** Update `AxelarAdapter` to pay gas to `AxelarGasService` before calling `AxelarGateway`. Fix `xdr` syntax in JS scripts to support contract deployment on Stellar Testnet.

**Tech Stack:** Rust (Soroban SDK), JavaScript (@stellar/stellar-sdk), Axelar GMP.

---

### Task 1: Enable AxelarAdapter Unit Tests

**Files:**
- Modify: `contracts/adapters/axelar/src/lib.rs`
- Test: `cargo test -p zolvency-axelar-adapter`

- [ ] **Step 1: Add mod test to lib.rs**

Add the module declaration at the end of the file.

```rust
// ... existing code ...

#[cfg(test)]
mod test;
```

- [ ] **Step 2: Run tests to verify they are now running (and failing)**

Run: `cargo test -p zolvency-axelar-adapter`
Expected: Tests should run. `test_send_flow` might fail because it expects gas payment which isn't implemented yet.

- [ ] **Step 3: Commit**

```bash
git add contracts/adapters/axelar/src/lib.rs
git commit -m "test: enable AxelarAdapter unit tests"
```

---

### Task 2: Implement Axelar Gas Payment

**Files:**
- Modify: `contracts/adapters/axelar/src/lib.rs`
- Test: `cargo test -p zolvency-axelar-adapter`

- [ ] **Step 1: Update call_axelar to pay gas**

Modify `call_axelar` to invoke the `GasService` before the `Gateway`.

```rust
    fn call_axelar(
        env: &Env,
        caller: Address,
        destination_chain: String,
        destination_address: String,
        payload: Bytes,
    ) -> Result<(), Error> {
        let gateway: Address = env.storage().instance().get(&DataKey::Gateway).ok_or(Error::NotInitialized)?;
        let gas_service: Address = env.storage().instance().get(&DataKey::GasService).ok_or(Error::NotInitialized)?;
        let gas_token_addr: Address = env.storage().instance().get(&DataKey::GasToken).ok_or(Error::NotInitialized)?;

        // 1. Pay for cross-chain gas
        let gas_token = AxelarGasToken {
            address: gas_token_addr,
            amount: Self::estimate_fee(env.clone(), destination_chain.clone()),
        };

        let _: Val = env.invoke_contract(
            &gas_service,
            &Symbol::new(env, "pay_gas"),
            (
                caller,
                destination_chain.clone(),
                destination_address.clone(),
                payload.clone(),
                env.current_contract_address(), // refund address
                gas_token,
                Bytes::new(env), // params
            )
                .into_val(env),
        );

        // 2. Call the Gateway
        let _: Val = env.invoke_contract(
            &gateway,
            &Symbol::new(env, "call_contract"),
            (
                env.current_contract_address(),
                destination_chain,
                destination_address,
                payload,
            )
                .into_val(env),
        );

        Ok(())
    }
```

- [ ] **Step 2: Update callers of call_axelar**

Ensure `caller` is passed correctly from `send_reputation` and `send_will_auth`.

- [ ] **Step 3: Run tests to verify success**

Run: `cargo test -p zolvency-axelar-adapter`
Expected: `test_send_flow` should PASS as it now finds both "gateway_call" and "gas_paid" events.

- [ ] **Step 4: Commit**

```bash
git add contracts/adapters/axelar/src/lib.rs
git commit -m "feat: implement Axelar gas payment"
```

---

### Task 3: Fix Deployment Script XDR Syntax

**Files:**
- Modify: `scripts/testnet_deploy_and_test.js`
- Test: `node scripts/testnet_deploy_and_test.js`

- [ ] **Step 1: Fix xdr.ContractIdPreimage instantiation**

Update line 89 in `scripts/testnet_deploy_and_test.js`.

```javascript
// OLD:
// sourceId: new xdr.ContractIdPreimage("contractIdPreimageFromAddress", ...)

// NEW:
sourceId: xdr.ContractIdPreimage.contractIdPreimageFromAddress(
    new xdr.ContractIdPreimageFromAddress({
        address: Address.fromString(deployerAddress).toScAddress(),
        salt: crypto.randomBytes(32)
    })
)
```

- [ ] **Step 2: Run deployment script**

Run: `node scripts/testnet_deploy_and_test.js` (Ensure your .env is set up or run it within a context that has it)
Expected: `XdrWriterError` is gone, and contract creation proceeds.

- [ ] **Step 3: Commit**

```bash
git add scripts/testnet_deploy_and_test.js
git commit -m "fix: update deprecated xdr syntax in deployment script"
```
