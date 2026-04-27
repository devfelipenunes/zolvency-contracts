# Axelar Interoperability Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable automatic "Push" of GitHub reputation from Stellar to an EVM chain using Axelar GMP.

**Architecture:** Modify `GithubIdentityContract` to call Axelar Gateway/GasService on `mint`/`update`. Create a `ZolvencyVerifier` Solidity contract on the destination chain.

**Tech Stack:** Soroban (Rust), Solidity, Axelar GMP.

---

### Task 1: [Stellar] Update Storage and Types
Update the configuration storage and types to support Axelar GMP settings.

**Files:**
- Modify: `github-identity/src/types.rs`
- Modify: `github-identity/src/storage.rs`

- [ ] **Step 1: Add AxelarConfig struct to types.rs**
```rust
#[derive(Clone, Debug)]
#[contracttype]
pub struct AxelarConfig {
    pub gateway: Address,
    pub gas_service: Address,
    pub gas_token: Address,
}
```

- [ ] **Step 2: Add storage functions to storage.rs**
```rust
pub fn get_axelar_config(env: &Env) -> Result<AxelarConfig, Error> {
    env.storage().instance().get(&DataKey::AxelarConfig).ok_or(Error::NotInitialized)
}

pub fn set_axelar_config(env: &Env, config: &AxelarConfig) {
    env.storage().instance().set(&DataKey::AxelarConfig, config);
}
```

- [ ] **Step 3: Commit**
```bash
git add github-identity/src/types.rs github-identity/src/storage.rs
git commit -m "feat(stellar): add axelar config storage and types"
```

---

### Task 2: [Stellar] Implement Axelar Gateway Interface
Create a Rust module or trait to interface with the Axelar Gateway and Gas Service contracts.

**Files:**
- Create: `github-identity/src/axelar.rs`
- Modify: `github-identity/src/lib.rs` (to register the new module)

- [ ] **Step 1: Define Axelar Gateway and Gas Service interfaces**
```rust
// github-identity/src/axelar.rs
use soroban_sdk::{contractimport, Address, Bytes, Env, String};

contractimport!(
    file = "target/wasm32-unknown-unknown/release/axelar_gateway.wasm"
); // Or use client directly if WASM is not available locally yet.
```
Actually, we can use `token::Client` and `env.invoke_contract` for Axelar Gateway if the WASM isn't easily reachable. I'll use the client approach.

```rust
pub struct AxelarClient<'a> {
    pub env: &'a Env,
    pub gateway: Address,
    pub gas_service: Address,
}

impl<'a> AxelarClient<'a> {
    pub fn call_contract(&self, destination_chain: String, destination_address: String, payload: Bytes) {
        self.env.invoke_contract::<(String, String, Bytes)>(
            &self.gateway,
            &Symbol::new(self.env, "call_contract"),
            (destination_chain, destination_address, payload).into_val(self.env),
        );
    }
}
```

- [ ] **Step 2: Commit**
```bash
git add github-identity/src/axelar.rs
git commit -m "feat(stellar): add axelar client module"
```

---

### Task 3: [Stellar] Update Mint and Update Token
Modify the `mint` and `update_token` functions to accept cross-chain parameters and call Axelar.

**Files:**
- Modify: `github-identity/src/lib.rs`

- [ ] **Step 1: Add cross-chain parameters to mint and update_token**
Accept `destination_chain: String`, `destination_address: String`, and `user_evm_address: Bytes`.

- [ ] **Step 2: Implement payload encoding (ABI-like)**
Since Soroban uses its own encoding, we need to match what EVM expects (ABI encoding). For a simple payload, we can use `(bytes32, uint8, bytes20)`.

- [ ] **Step 3: Call Axelar Gas Service and Gateway**
```rust
// Pay gas to Axelar Gas Service
let gas_token = token::Client::new(&env, &axelar_config.gas_token);
gas_token.transfer(&caller, &axelar_config.gas_service, &1000000); // Fixed or calculated amount

// Call Gateway
gateway.call_contract(destination_chain, destination_address, payload);
```

- [ ] **Step 4: Commit**
```bash
git add github-identity/src/lib.rs
git commit -m "feat(stellar): push reputation to axelar on mint/update"
```

---

### Task 4: [EVM] Create ZolvencyVerifier Contract
Implement the receiver contract on the EVM side using Solidity and AxelarExecutable.

**Files:**
- Create: `contracts/ZolvencyVerifier.sol`

- [ ] **Step 1: Implement the Verifier contract**
```solidity
// contracts/ZolvencyVerifier.sol
pragma solidity ^0.8.0;

import { IAxelarGateway } from "@axelar-network/axelar-gmp-sdk-solidity/contracts/interfaces/IAxelarGateway.sol";
import { AxelarExecutable } from "@axelar-network/axelar-gmp-sdk-solidity/contracts/executable/AxelarExecutable.sol";

contract ZolvencyVerifier is AxelarExecutable {
    struct Reputation {
        bytes32 externalId;
        uint8 tier;
    }

    mapping(address => Reputation) public reputations;
    string public sourceStellarAddress;

    constructor(address _gateway, string memory _sourceAddress) AxelarExecutable(_gateway) {
        sourceStellarAddress = _sourceAddress;
    }

    function _execute(string calldata sourceChain, string calldata sourceAddress, bytes calldata payload) internal override {
        require(keccak256(bytes(sourceChain)) == keccak256(bytes("stellar")), "Invalid source chain");
        require(keccak256(bytes(sourceAddress)) == keccak256(bytes(sourceStellarAddress)), "Invalid source address");

        (bytes32 externalId, uint8 tier, address user) = abi.decode(payload, (bytes32, uint8, address));
        reputations[user] = Reputation(externalId, tier);
    }
}
```

- [ ] **Step 2: Commit**
```bash
git add contracts/ZolvencyVerifier.sol
git commit -m "feat(evm): add zolvency verifier contract"
```

---

### Task 5: [Testing] Integration Test on Stellar
Mock the Axelar components and verify that `mint` correctly triggers a call to the "Gateway".

**Files:**
- Modify: `github-identity/src/test.rs`

- [ ] **Step 1: Create Mock Axelar Gateway**
Implement a contract that records calls to `call_contract`.

- [ ] **Step 2: Write test for cross-chain mint**
Assert that the Mock Gateway received the expected payload.

- [ ] **Step 3: Commit**
```bash
git add github-identity/src/test.rs
git commit -m "test(stellar): verify cross-chain push during mint"
```
