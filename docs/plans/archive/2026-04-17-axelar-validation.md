# Axelar Adapter Validation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Verify the end-to-end reputation push using the new modular Axelar Adapter.

**Architecture:** Deploy the modular Stellar contract and the Axelar-specific EVM verifier, then link them.

**Tech Stack:** Soroban, Foundry, Axelar Sepolia/Stellar Testnet.

---

### Task 1: [EVM] Deploy Axelar Verifier
Deploy the renamed verifier contract to Sepolia.

**Files:**
- Create: `packages/evm/script/DeployAxelarVerifier.s.sol`

- [ ] **Step 1: Create the deployment script**
```solidity
// packages/evm/script/DeployAxelarVerifier.s.sol
pragma solidity ^0.8.0;
import "forge-std/Script.sol";
import "../src/ZolvencyVerifierAxelar.sol";

contract DeployAxelarVerifier is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("EVM_PRIVATE_KEY");
        address gateway = 0xe432150cce91c13a887f7D836923d5597adD8E31; // Axelar Sepolia Gateway
        string memory stellarSource = vm.envString("STELLAR_IDENTITY_ADDRESS");

        vm.startBroadcast(deployerPrivateKey);
        ZolvencyVerifierAxelar verifier = new ZolvencyVerifierAxelar(gateway, stellarSource);
        console.log("ZolvencyVerifierAxelar deployed to:", address(verifier));
        vm.stopBroadcast();
    }
}
```

- [ ] **Step 2: Run deploy (Simulation)**
Run: `cd packages/evm && forge script script/DeployAxelarVerifier.s.sol --rpc-url sepolia`

- [ ] **Step 3: Commit**
```bash
git add packages/evm/script/DeployAxelarVerifier.s.sol
git commit -m "test(evm): add axelar verifier deploy script"
```

---

### Task 2: [Stellar] Configure Axelar Adapter
Activate the Axelar protocol in the modular Identity contract.

**Files:**
- Create: `packages/stellar/contracts/github-identity/scripts/activate_axelar.sh`

- [ ] **Step 1: Create the activation script**
This script will call `set_active_protocol` on the Stellar contract.

- [ ] **Step 2: Run activation on Testnet**
(Assuming the contract is already deployed, otherwise deploy first).

- [ ] **Step 3: Commit**
```bash
git add packages/stellar/contracts/github-identity/scripts/activate_axelar.sh
git commit -m "test(stellar): add axelar activation script"
```

---

### Task 3: [Integration] Full Flow Test
Trigger a mint on Stellar and verify the receipt on Axelarscan/EVM.

- [ ] **Step 1: Call mint with CrossChainParams**
- [ ] **Step 2: Monitor Axelarscan for the message**
- [ ] **Step 3: Verify the mapping in ZolvencyVerifierAxelar**
