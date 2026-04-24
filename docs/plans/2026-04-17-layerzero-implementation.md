# LayerZero V2 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement LayerZero V2 as a secondary interoperability provider for Zolvency.

**Architecture:** Maintain the Adapter Pattern. Create a `LayerZeroAdapter` on Stellar and an `OApp` Verifier on EVM.

**Tech Stack:** Soroban, Solidity, LayerZero V2 (OApp).

---

### Task 1: [Stellar] Implement LayerZero Adapter
Create the adapter that interfaces with the LayerZero Endpoint on Stellar.

**Files:**
- Create/Modify: `packages/stellar/contracts/github-identity/src/layerzero_adapter.rs`

- [ ] **Step 1: Define LayerZero Endpoint Interface**
Import the necessary wasm/client for LayerZero Endpoint V2.

- [ ] **Step 2: Implement MessengerTrait for LayerZeroAdapter**
```rust
pub struct LayerZeroAdapter;

impl MessengerTrait for LayerZeroAdapter {
    fn send_reputation(...) {
        // 1. Encode params for LZ (Options, MessagingFee)
        // 2. Call endpoint.send()
    }
}
```

- [ ] **Step 3: Commit**
```bash
git add packages/stellar/contracts/github-identity/src/layerzero_adapter.rs
git commit -m "feat(stellar): implement LayerZero V2 adapter"
```

---

### Task 2: [EVM] Implement OApp Verifier
Finalize the Solidity verifier using the LayerZero OApp standard.

**Files:**
- Modify: `packages/evm/src/ZolvencyVerifierLayerZero.sol`

- [ ] **Step 1: Inherit from OApp**
- [ ] **Step 2: Implement _lzReceive**
```solidity
function _lzReceive(
    Origin calldata _origin,
    bytes32 _guid,
    bytes calldata _message,
    address _executor,
    bytes calldata _extraData
) internal override {
    (bytes32 externalId, uint8 tier, address user) = abi.decode(_message, (bytes32, uint8, address));
    reputations[user] = Reputation(externalId, tier);
}
```

- [ ] **Step 3: Commit**
```bash
git add packages/evm/src/ZolvencyVerifierLayerZero.sol
git commit -m "feat(evm): finalize LayerZero OApp verifier"
```

---

### Task 3: [Stellar] Update Dispatcher
Ensure the core contract correctly routes to the LayerZero adapter.

- [ ] **Step 1: Update lib.rs to include LayerZeroAdapter**
- [ ] **Step 2: Verify Build**
- [ ] **Step 3: Commit**
```bash
git add packages/stellar/contracts/github-identity/src/lib.rs
git commit -m "feat(stellar): enable LayerZero dispatch in core contract"
```
